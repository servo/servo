/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::default::Default;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::intrinsics;
use std::mem;
use std::sync::Arc;

use cssparser::{Parser, Color, RGBA, AtRuleParser, DeclarationParser,
                DeclarationListParser, parse_important, ToCss};
use url::Url;
use util::geometry::Au;
use util::logical_geometry::{LogicalMargin, PhysicalSide, WritingMode};
use euclid::SideOffsets2D;
use euclid::size::Size2D;
use fnv::FnvHasher;

use computed_values;
use parser::{ParserContext, log_css_error};
use selectors::matching::DeclarationBlock;
use stylesheets::Origin;
use values::computed::{self, ToComputedValue};
use values::specified::{Length, BorderStyle};

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
    def __init__(self, name, derived_from=None, custom_cascade=False, experimental=False):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.style_struct = THIS_STYLE_STRUCT
        self.experimental = experimental
        self.custom_cascade = custom_cascade
        if derived_from is None:
            self.derived_from = None
        else:
            self.derived_from = [ to_rust_ident(name) for name in derived_from ]

class Shorthand(object):
    def __init__(self, name, sub_properties, experimental=False):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.derived_from = None
        self.experimental = experimental
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
    use cssparser::Parser;
    use parser::ParserContext;
    use values::specified;

    <%def name="raw_longhand(name, derived_from=None, custom_cascade=False, experimental=False)">
    <%
        if derived_from is not None:
            derived_from = derived_from.split()

        property = Longhand(name,
                            derived_from=derived_from,
                            custom_cascade=custom_cascade,
                            experimental=experimental)
        THIS_STYLE_STRUCT.longhands.append(property)
        LONGHANDS.append(property)
        LONGHANDS_BY_NAME[name] = property

        if derived_from is not None:
            for name in derived_from:
                DERIVED_LONGHANDS.setdefault(name, []).append(property)
    %>
        pub mod ${property.ident} {
            #![allow(unused_imports)]
            % if derived_from is None:
                use cssparser::Parser;
                use parser::ParserContext;
                use properties::{CSSWideKeyword, DeclaredValue};
            % endif
            use properties::longhands;
            use properties::property_bit_field::PropertyBitField;
            use properties::{ComputedValues, PropertyDeclaration};
            use std::sync::Arc;
            use values::computed::ToComputedValue;
            use values::{computed, specified};
            ${caller.body()}
            #[allow(unused_variables)]
            pub fn cascade_property(declaration: &PropertyDeclaration,
                                    style: &mut ComputedValues,
                                    inherited_style: &ComputedValues,
                                    context: &computed::Context,
                                    seen: &mut PropertyBitField,
                                    cacheable: &mut bool) {
                let declared_value = match *declaration {
                    PropertyDeclaration::${property.camel_case}(ref declared_value) => {
                        declared_value
                    }
                    _ => panic!("entered the wrong cascade_property() implementation"),
                };
                % if property.derived_from is None:
                    if seen.get_${property.ident}() {
                        return
                    }
                    seen.set_${property.ident}();
                    let computed_value = match *declared_value {
                        DeclaredValue::SpecifiedValue(ref specified_value) => {
                            specified_value.to_computed_value(&context)
                        }
                        DeclaredValue::Initial => get_initial_value(),
                        DeclaredValue::Inherit => {
                            // This is a bit slow, but this is rare so it shouldn't
                            // matter.
                            //
                            // FIXME: is it still?
                            *cacheable = false;
                            inherited_style.${THIS_STYLE_STRUCT.ident}
                                           .${property.ident}
                                           .clone()
                        }
                    };
                    Arc::make_unique(&mut style.${THIS_STYLE_STRUCT.ident}).${property.ident} =
                        computed_value;

                    % if custom_cascade:
                        cascade_property_custom(&computed_value,
                                                declaration,
                                                style,
                                                inherited_style,
                                                context,
                                                seen,
                                                cacheable);
                    % endif
                % else:
                    // Do not allow stylesheets to set derived properties.
                % endif
            }
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

    <%def name="longhand(name, derived_from=None, custom_cascade=False, experimental=False)">
        <%self:raw_longhand name="${name}" derived_from="${derived_from}"
                custom_cascade="${custom_cascade}" experimental="${experimental}">
            ${caller.body()}
            % if derived_from is None:
                pub fn parse_specified(context: &ParserContext, input: &mut Parser)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    parse(context, input).map(DeclaredValue::SpecifiedValue)
                }
            % endif
        </%self:raw_longhand>
    </%def>

    <%def name="single_keyword_computed(name, values, custom_cascade=False, experimental=False)">
        <%self:longhand name="${name}" custom_cascade="${custom_cascade}"
            experimental="${experimental}">
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
            use values::computed::ComputedValueAsSpecified;
            impl ComputedValueAsSpecified for SpecifiedValue {}
        </%self:single_keyword_computed>
    </%def>

    <%def name="predefined_type(name, type, initial_value, parse_method='parse')">
        <%self:longhand name="${name}">
            #[allow(unused_imports)]
            use util::geometry::Au;
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
        ${predefined_type("border-%s-style" % side, "BorderStyle", "specified::BorderStyle::none")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        <%self:longhand name="border-${side}-width">
            use cssparser::ToCss;
            use std::fmt;
            use util::geometry::Au;
            use values::computed::Context;

            impl ToCss for SpecifiedValue {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    self.0.to_css(dest)
                }
            }

            #[inline]
            pub fn parse(_context: &ParserContext, input: &mut Parser)
                                   -> Result<SpecifiedValue, ()> {
                specified::parse_border_width(input).map(SpecifiedValue)
            }
            #[derive(Clone, PartialEq)]
            pub struct SpecifiedValue(pub specified::Length);
            pub mod computed_value {
                use util::geometry::Au;
                pub type T = Au;
            }
            #[inline] pub fn get_initial_value() -> computed_value::T {
                Au::from_px(3)  // medium
            }

            impl ToComputedValue for SpecifiedValue {
                type ComputedValue = computed_value::T;

                #[inline]
                fn to_computed_value(&self, context: &Context) -> computed_value::T {
                    if !context.border_${side}_present {
                        Au(0)
                    } else {
                        self.0.to_computed_value(context)
                    }
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
        use cssparser::ToCss;
        use std::fmt;
        use util::geometry::Au;
        use values::computed::Context;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            specified::parse_border_width(input).map(SpecifiedValue)
        }
        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(pub specified::Length);
        pub mod computed_value {
            use util::geometry::Au;
            pub type T = Au;
        }
        pub use super::border_top_width::get_initial_value;
        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                if !context.outline_style_present {
                    Au(0)
                } else {
                    self.0.to_computed_value(context)
                }
            }
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
    <%self:longhand name="display" custom_cascade="True">
        <%
            values = """inline block inline-block
                table inline-table table-row-group table-header-group table-footer-group
                table-row table-column-group table-column table-cell table-caption
                list-item flex
                none
            """.split()
            experimental_values = set("flex".split())
        %>
        pub use self::computed_value::T as SpecifiedValue;
        use values::computed::Context;

        pub mod computed_value {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug, HeapSizeOf)]
            #[derive(Deserialize, Serialize)]
            pub enum T {
                % for value in values:
                    ${to_rust_ident(value)},
                % endfor
            }

            impl ::cssparser::ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
                where W: ::std::fmt::Write {
                    match self {
                        % for value in values:
                            &T::${to_rust_ident(value)} => dest.write_str("${value}"),
                        % endfor
                    }
                }
            }
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::T::${to_rust_ident(values[0])}
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                % for value in values[:-1]:
                    "${value}" => {
                        % if value in experimental_values:
                            if !::util::opts::experimental_enabled() { return Err(()) }
                        % endif
                        Ok(computed_value::T::${to_rust_ident(value)})
                    },
                % endfor
                % for value in values[-1:]:
                    "${value}" => {
                        % if value in experimental_values:
                            if !::util::opts::experimental_enabled() { return Err(()) }
                        % endif
                        Ok(computed_value::T::${to_rust_ident(value)})
                    }
                % endfor
                _ => Err(())
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                use self::computed_value::T;
    //            if context.is_root_element && value == list_item {
    //                return block
    //            }
                if context.positioned || context.floated || context.is_root_element {
                    match *self {
                        T::inline_table => T::table,
                        T::inline | T::inline_block |
                        T::table_row_group | T::table_column |
                        T::table_column_group | T::table_header_group |
                        T::table_footer_group | T::table_row | T::table_cell |
                        T::table_caption
                        => T::block,
                        _ => *self,
                    }
                } else {
                    *self
                }
            }
        }

        fn cascade_property_custom(computed_value: &computed_value::T,
                                   _declaration: &PropertyDeclaration,
                                   style: &mut ComputedValues,
                                   _inherited_style: &ComputedValues,
                                   context: &computed::Context,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool) {
            Arc::make_unique(&mut style.box_)._servo_display_for_hypothetical_box =
                longhands::_servo_display_for_hypothetical_box::derive_from_display(
                    *computed_value,
                    &context);
            Arc::make_unique(&mut style.inheritedtext)._servo_text_decorations_in_effect =
                longhands::_servo_text_decorations_in_effect::derive_from_display(*computed_value,
                                                                                  &context);
        }
    </%self:longhand>

    ${single_keyword("position", "static absolute relative fixed")}
    ${single_keyword("float", "none left right")}
    ${single_keyword("clear", "none left right both")}

    <%self:longhand name="-servo-display-for-hypothetical-box" derived_from="display">
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
        use values::computed::ComputedValueAsSpecified;

        impl ComputedValueAsSpecified for SpecifiedValue {}
        pub type SpecifiedValue = computed_value::T;
        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;

            #[derive(PartialEq, Clone, Eq, Copy, Debug, HeapSizeOf)]
            pub enum T {
                Auto,
                Number(i32),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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

    ${single_keyword("direction", "ltr rtl")}

    // CSS 2.1, Section 10 - Visual formatting model details

    ${switch_to_style_struct("Box")}

    ${predefined_type("width", "LengthOrPercentageOrAuto",
                      "computed::LengthOrPercentageOrAuto::Auto",
                      "parse_non_negative")}

    ${predefined_type("height", "LengthOrPercentageOrAuto",
                      "computed::LengthOrPercentageOrAuto::Auto",
                      "parse_non_negative")}

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
        use std::fmt;
        use values::CSSFloat;
        use values::computed::Context;

        #[derive(Clone, PartialEq, Copy)]
        pub enum SpecifiedValue {
            Normal,
            Length(specified::Length),
            Number(CSSFloat),
            Percentage(CSSFloat),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
            use cssparser::Token;
            use std::ascii::AsciiExt;
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
            use std::fmt;
            use util::geometry::Au;
            use values::CSSFloat;
            #[derive(PartialEq, Copy, Clone, HeapSizeOf)]
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
        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    computed_value::T::Normal => dest.write_str("normal"),
                    computed_value::T::Length(length) => length.to_css(dest),
                    computed_value::T::Number(number) => write!(dest, "{}", number),
                }
            }
        }
         #[inline]
        pub fn get_initial_value() -> computed_value::T { computed_value::T::Normal }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T::Normal,
                    SpecifiedValue::Length(value) => {
                        computed_value::T::Length(value.to_computed_value(context))
                    }
                    SpecifiedValue::Number(value) => computed_value::T::Number(value),
                    SpecifiedValue::Percentage(value) => {
                        let fr = specified::Length::FontRelative(specified::FontRelativeLength::Em(value));
                        computed_value::T::Length(fr.to_computed_value(context))
                    }
                }
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("Box")}

    <%self:longhand name="vertical-align">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;

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
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
            input.try(specified::LengthOrPercentage::parse)
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
            use std::fmt;
            use util::geometry::Au;
            use values::CSSFloat;
            #[allow(non_camel_case_types)]
            #[derive(PartialEq, Copy, Clone, HeapSizeOf)]
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
            impl ::cssparser::ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        % for keyword in vertical_align_keywords:
                            T::${to_rust_ident(keyword)} => dest.write_str("${keyword}"),
                        % endfor
                        T::Length(value) => value.to_css(dest),
                        T::Percentage(percentage) => write!(dest, "{}%", percentage * 100.),
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { computed_value::T::baseline }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    % for keyword in vertical_align_keywords:
                        SpecifiedValue::${to_rust_ident(keyword)} => {
                            computed_value::T::${to_rust_ident(keyword)}
                        }
                    % endfor
                    SpecifiedValue::LengthOrPercentage(value) => {
                        match value.to_computed_value(context) {
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
        }
    </%self:longhand>


    // CSS 2.1, Section 11 - Visual effects

    // FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
    <%self:single_keyword_computed name="overflow-x" values="visible hidden scroll auto">
        use values::computed::Context;

        pub fn compute_with_other_overflow_direction(value: SpecifiedValue,
                                                     other_direction: SpecifiedValue)
                                                     -> computed_value::T {
            // CSS-OVERFLOW 3 states "Otherwise, if one cascaded values is one of the scrolling
            // values and the other is `visible`, then computed values are the cascaded values with
            // `visible` changed to `auto`."
            match (value, other_direction) {
                (SpecifiedValue::visible, SpecifiedValue::hidden) |
                (SpecifiedValue::visible, SpecifiedValue::scroll) |
                (SpecifiedValue::visible, SpecifiedValue::auto) => computed_value::T::auto,
                _ => value,
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                compute_with_other_overflow_direction(*self, context.overflow_y.0)
            }
        }
    </%self:single_keyword_computed>

    // FIXME(pcwalton, #2742): Implement scrolling for `scroll` and `auto`.
    <%self:longhand name="overflow-y">
        use super::overflow_x;
        use values::computed::Context;

        use cssparser::ToCss;
        use std::fmt;

        pub use self::computed_value::T as SpecifiedValue;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        pub mod computed_value {
            #[derive(Clone, Copy, PartialEq, HeapSizeOf)]
            pub struct T(pub super::super::overflow_x::computed_value::T);
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                let computed_value::T(this) = *self;
                computed_value::T(overflow_x::compute_with_other_overflow_direction(
                        this,
                        context.overflow_x))
            }
        }

        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(overflow_x::get_initial_value())
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            overflow_x::parse(context, input).map(|value| SpecifiedValue(value))
        }
    </%self:longhand>


    ${switch_to_style_struct("InheritedBox")}

    // TODO: collapse. Well, do tables first.
    ${single_keyword("visibility", "visible hidden")}

    // CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

    ${switch_to_style_struct("Box")}

    <%self:longhand name="content">
        use cssparser::Token;
        use std::ascii::AsciiExt;
        use values::computed::ComputedValueAsSpecified;

        use super::list_style_type;

        pub use self::computed_value::T as SpecifiedValue;
        pub use self::computed_value::ContentItem;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        pub mod computed_value {
            use super::super::list_style_type;

            use cssparser::{self, ToCss};
            use std::fmt;

            #[derive(PartialEq, Eq, Clone, HeapSizeOf)]
            pub enum ContentItem {
                /// Literal string content.
                String(String),
                /// `counter(name, style)`.
                Counter(String, list_style_type::computed_value::T),
                /// `counters(name, separator, style)`.
                Counters(String, String, list_style_type::computed_value::T),
                /// `open-quote`.
                OpenQuote,
                /// `close-quote`.
                CloseQuote,
                /// `no-open-quote`.
                NoOpenQuote,
                /// `no-close-quote`.
                NoCloseQuote,
            }

            impl ToCss for ContentItem {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match self {
                        &ContentItem::String(ref s) => {
                            cssparser::serialize_string(&**s, dest)
                        }
                        &ContentItem::Counter(ref s, ref list_style_type) => {
                            try!(dest.write_str("counter("));
                            try!(cssparser::serialize_identifier(&**s, dest));
                            try!(dest.write_str(", "));
                            try!(list_style_type.to_css(dest));
                            dest.write_str(")")
                        }
                        &ContentItem::Counters(ref s, ref separator, ref list_style_type) => {
                            try!(dest.write_str("counter("));
                            try!(cssparser::serialize_identifier(&**s, dest));
                            try!(dest.write_str(", "));
                            try!(cssparser::serialize_string(&**separator, dest));
                            try!(dest.write_str(", "));
                            try!(list_style_type.to_css(dest));
                            dest.write_str(")")
                        }
                        &ContentItem::OpenQuote => dest.write_str("open-quote"),
                        &ContentItem::CloseQuote => dest.write_str("close-quote"),
                        &ContentItem::NoOpenQuote => dest.write_str("no-open-quote"),
                        &ContentItem::NoCloseQuote => dest.write_str("no-close-quote"),
                    }
                }
            }

            #[allow(non_camel_case_types)]
            #[derive(PartialEq, Eq, Clone, HeapSizeOf)]
            pub enum T {
                normal,
                none,
                Content(Vec<ContentItem>),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match self {
                        &T::normal => dest.write_str("normal"),
                        &T::none => dest.write_str("none"),
                        &T::Content(ref content) => {
                            let mut iter = content.iter();
                            try!(iter.next().unwrap().to_css(dest));
                            for c in iter {
                                try!(dest.write_str(" "));
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

        pub fn counter_name_is_illegal(name: &str) -> bool {
            name.eq_ignore_ascii_case("none") || name.eq_ignore_ascii_case("inherit") ||
                name.eq_ignore_ascii_case("initial")
        }

        // normal | none | [ <string> | <counter> | open-quote | close-quote | no-open-quote |
        // no-close-quote ]+
        // TODO: <uri>, attr(<identifier>)
        pub fn parse(context: &ParserContext, input: &mut Parser)
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
                        content.push(ContentItem::String(value.into_owned()))
                    }
                    Ok(Token::Function(name)) => {
                        content.push(try!(match_ignore_ascii_case! { name,
                            "counter" => input.parse_nested_block(|input| {
                                let name = try!(input.expect_ident()).into_owned();
                                let style = input.try(|input| {
                                    try!(input.expect_comma());
                                    list_style_type::parse(context, input)
                                }).unwrap_or(list_style_type::computed_value::T::decimal);
                                Ok(ContentItem::Counter(name, style))
                            }),
                            "counters" => input.parse_nested_block(|input| {
                                let name = try!(input.expect_ident()).into_owned();
                                try!(input.expect_comma());
                                let separator = try!(input.expect_string()).into_owned();
                                let style = input.try(|input| {
                                    try!(input.expect_comma());
                                    list_style_type::parse(context, input)
                                }).unwrap_or(list_style_type::computed_value::T::decimal);
                                Ok(ContentItem::Counters(name, separator, style))
                            })
                            _ => return Err(())
                        }));
                    }
                    Ok(Token::Ident(ident)) => {
                        match_ignore_ascii_case! { ident,
                            "open-quote" => content.push(ContentItem::OpenQuote),
                            "close-quote" => content.push(ContentItem::CloseQuote),
                            "no-open-quote" => content.push(ContentItem::NoOpenQuote),
                            "no-close-quote" => content.push(ContentItem::NoCloseQuote)
                            _ => return Err(())
                        }
                    }
                    Err(_) => break,
                    _ => return Err(())
                }
            }
            if !content.is_empty() {
                Ok(SpecifiedValue::Content(content))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    ${new_style_struct("List", is_inherited=True)}

    ${single_keyword("list-style-position", "outside inside")}

    // TODO(pcwalton): Implement the full set of counter styles per CSS-COUNTER-STYLES [1] 6.1:
    //
    //     decimal-leading-zero, armenian, upper-armenian, lower-armenian, georgian, lower-roman,
    //     upper-roman
    //
    // [1]: http://dev.w3.org/csswg/css-counter-styles/
    ${single_keyword("list-style-type", """
        disc none circle square decimal arabic-indic bengali cambodian cjk-decimal devanagari
        gujarati gurmukhi kannada khmer lao malayalam mongolian myanmar oriya persian telugu thai
        tibetan lower-alpha upper-alpha cjk-earthly-branch cjk-heavenly-stem lower-greek hiragana
        hiragana-iroha katakana katakana-iroha disclosure-open disclosure-closed
    """)}

    <%self:longhand name="list-style-image">
        use cssparser::{ToCss, Token};
        use std::fmt;
        use url::Url;
        use values::computed::Context;

        #[derive(Clone, PartialEq, Eq)]
        pub enum SpecifiedValue {
            None,
            Url(Url),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::None => dest.write_str("none"),
                    SpecifiedValue::Url(ref url) => {
                        Token::Url(url.to_string().into()).to_css(dest)
                    }
                }
            }
        }

        pub mod computed_value {
            use cssparser::{ToCss, Token};
            use std::fmt;
            use url::Url;

            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Url>);

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match self.0 {
                        None => dest.write_str("none"),
                        Some(ref url) => Token::Url(url.to_string().into()).to_css(dest)
                    }
                }
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::None => computed_value::T(None),
                    SpecifiedValue::Url(ref url) => computed_value::T(Some(url.clone())),
                }
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
            computed_value::T(None)
        }
    </%self:longhand>

    <%self:longhand name="quotes">
        use std::borrow::Cow;
        use std::fmt;
        use values::computed::ComputedValueAsSpecified;

        use cssparser::{ToCss, Token};

        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<(String,String)>);
        }

        impl ComputedValueAsSpecified for SpecifiedValue {}

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut first = true;
                for pair in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(Token::QuotedString(Cow::from(&*pair.0)).to_css(dest));
                    try!(dest.write_str(" "));
                    try!(Token::QuotedString(Cow::from(&*pair.1)).to_css(dest));
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![
                ("\u{201c}".to_owned(), "\u{201d}".to_owned()),
                ("\u{2018}".to_owned(), "\u{2019}".to_owned()),
            ])
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Vec::new()))
            }

            let mut quotes = Vec::new();
            loop {
                let first = match input.next() {
                    Ok(Token::QuotedString(value)) => value.into_owned(),
                    Ok(_) => return Err(()),
                    Err(()) => break,
                };
                let second = match input.next() {
                    Ok(Token::QuotedString(value)) => value.into_owned(),
                    _ => return Err(()),
                };
                quotes.push((first, second))
            }
            if !quotes.is_empty() {
                Ok(SpecifiedValue(quotes))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    ${new_style_struct("Counters", is_inherited=False)}

    <%self:longhand name="counter-increment">
        use std::fmt;
        use super::content;
        use values::computed::ComputedValueAsSpecified;

        use cssparser::{ToCss, Token};
        use std::borrow::{Cow, ToOwned};

        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<(String,i32)>);
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        impl ComputedValueAsSpecified for SpecifiedValue {}

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut first = true;
                for pair in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(Token::QuotedString(Cow::from(&*pair.0)).to_css(dest));
                    try!(write!(dest, " {}", pair.1));
                }
                Ok(())
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Vec::new()))
            }

            let mut counters = Vec::new();
            loop {
                let counter_name = match input.next() {
                    Ok(Token::Ident(ident)) => (*ident).to_owned(),
                    Ok(_) => return Err(()),
                    Err(_) => break,
                };
                if content::counter_name_is_illegal(&counter_name) {
                    return Err(())
                }
                let counter_delta = input.try(|input| input.expect_integer()).unwrap_or(1) as i32;
                counters.push((counter_name, counter_delta))
            }

            if !counters.is_empty() {
                Ok(SpecifiedValue(counters))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    <%self:longhand name="counter-reset">
        pub use super::counter_increment::{SpecifiedValue, computed_value, get_initial_value};
        pub use super::counter_increment::{parse};
    </%self:longhand>

    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    ${new_style_struct("Background", is_inherited=False)}
    ${predefined_type(
        "background-color", "CSSColor",
        "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */")}

    <%self:longhand name="background-image">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;
        use values::specified::Image;

        pub mod computed_value {
            use values::computed;
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<computed::Image>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("none"),
                    Some(computed::Image::Url(ref url)) =>
                        ::cssparser::Token::Url(url.to_string().into()).to_css(dest),
                    Some(computed::Image::LinearGradient(ref gradient)) =>
                        gradient.to_css(dest)
                }
            }
        }

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(pub Option<Image>);

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue(Some(ref image)) => image.to_css(dest),
                    SpecifiedValue(None) => dest.write_str("none"),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }
        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue(None))
            } else {
                Ok(SpecifiedValue(Some(try!(Image::parse(context, input)))))
            }
        }
        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue(None) => computed_value::T(None),
                    SpecifiedValue(Some(ref image)) =>
                        computed_value::T(Some(image.to_computed_value(context))),
                }
            }
        }
    </%self:longhand>

    <%self:longhand name="background-position">
            use cssparser::ToCss;
            use std::fmt;
            use values::computed::Context;

            pub mod computed_value {
                use values::computed::LengthOrPercentage;

                #[derive(PartialEq, Copy, Clone, Debug, HeapSizeOf)]
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
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    try!(self.horizontal.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(self.vertical.to_css(dest));
                    Ok(())
                }
            }

            impl ToCss for computed_value::T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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

            impl ToComputedValue for SpecifiedValue {
                type ComputedValue = computed_value::T;

                #[inline]
                fn to_computed_value(&self, context: &Context) -> computed_value::T {
                    computed_value::T {
                        horizontal: self.horizontal.to_computed_value(context),
                        vertical: self.vertical.to_computed_value(context),
                    }
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

    ${single_keyword("background-clip", "border-box padding-box content-box")}

    ${single_keyword("background-origin", "padding-box border-box content-box")}

    <%self:longhand name="background-size">
        use cssparser::{ToCss, Token};
        use std::ascii::AsciiExt;
        use std::fmt;
        use values::computed::Context;

        pub mod computed_value {
            use values::computed::LengthOrPercentageOrAuto;

            #[derive(PartialEq, Clone, Debug, HeapSizeOf)]
            pub struct ExplicitSize {
                pub width: LengthOrPercentageOrAuto,
                pub height: LengthOrPercentageOrAuto,
            }

            #[derive(PartialEq, Clone, Debug, HeapSizeOf)]
            pub enum T {
                Explicit(ExplicitSize),
                Cover,
                Contain,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    computed_value::T::Explicit(ref size) => size.to_css(dest),
                    computed_value::T::Cover => dest.write_str("cover"),
                    computed_value::T::Contain => dest.write_str("contain"),
                }
            }
        }

        #[derive(Clone, PartialEq, Debug)]
        pub struct SpecifiedExplicitSize {
            pub width: specified::LengthOrPercentageOrAuto,
            pub height: specified::LengthOrPercentageOrAuto,
        }

        impl ToCss for SpecifiedExplicitSize {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.width.to_css(dest));
                try!(dest.write_str(" "));
                self.height.to_css(dest)
            }
        }

        impl ToCss for computed_value::ExplicitSize {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.width.to_css(dest));
                try!(dest.write_str(" "));
                self.height.to_css(dest)
            }
        }


        #[derive(Clone, PartialEq, Debug)]
        pub enum SpecifiedValue {
            Explicit(SpecifiedExplicitSize),
            Cover,
            Contain,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Explicit(ref size) => size.to_css(dest),
                    SpecifiedValue::Cover => dest.write_str("cover"),
                    SpecifiedValue::Contain => dest.write_str("contain"),
                }
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &computed::Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Explicit(ref size) => {
                        computed_value::T::Explicit(computed_value::ExplicitSize {
                            width: size.width.to_computed_value(context),
                            height: size.height.to_computed_value(context),
                        })
                    }
                    SpecifiedValue::Cover => computed_value::T::Cover,
                    SpecifiedValue::Contain => computed_value::T::Contain,
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Explicit(computed_value::ExplicitSize {
                width: computed::LengthOrPercentageOrAuto::Auto,
                height: computed::LengthOrPercentageOrAuto::Auto,
            })
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let width;
            if let Ok(value) = input.try(|input| {
                match input.next() {
                    Err(_) => Err(()),
                    Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("cover") => {
                        Ok(SpecifiedValue::Cover)
                    }
                    Ok(Token::Ident(ref ident)) if ident.eq_ignore_ascii_case("contain") => {
                        Ok(SpecifiedValue::Contain)
                    }
                    Ok(_) => Err(()),
                }
            }) {
                return Ok(value)
            } else {
                width = try!(specified::LengthOrPercentageOrAuto::parse(input))
            }

            let height;
            if let Ok(value) = input.try(|input| {
                match input.next() {
                    Err(_) => Ok(specified::LengthOrPercentageOrAuto::Auto),
                    Ok(_) => Err(()),
                }
            }) {
                height = value
            } else {
                height = try!(specified::LengthOrPercentageOrAuto::parse(input));
            }

            Ok(SpecifiedValue::Explicit(SpecifiedExplicitSize {
                width: width,
                height: height,
            }))
        }
    </%self:longhand>

    ${new_style_struct("Color", is_inherited=True)}

    <%self:raw_longhand name="color">
        use cssparser::{Color, RGBA};
        use values::computed::Context;
        use values::specified::{CSSColor, CSSRGBA};

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _context: &Context) -> computed_value::T {
                self.parsed
            }
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
        use self::computed_value::FontFamily;
        use string_cache::Atom;
        use values::computed::ComputedValueAsSpecified;
        pub use self::computed_value::T as SpecifiedValue;

        impl ComputedValueAsSpecified for SpecifiedValue {}
        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;
            use string_cache::Atom;

            #[derive(PartialEq, Eq, Clone, Hash, HeapSizeOf)]
            pub enum FontFamily {
                FamilyName(Atom),
                // Generic
//                Serif,
//                SansSerif,
//                Cursive,
//                Fantasy,
//                Monospace,
            }
            impl FontFamily {
                #[inline]
                pub fn name(&self) -> &str {
                    match *self {
                        FontFamily::FamilyName(ref name) => name.as_slice(),
                    }
                }
            }
            impl ToCss for FontFamily {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match self {
                        &FontFamily::FamilyName(ref name) => dest.write_str(name.as_slice()),
                    }
                }
            }
            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    let mut iter = self.0.iter();
                    try!(iter.next().unwrap().to_css(dest));
                    for family in iter {
                        try!(dest.write_str(", "));
                        try!(family.to_css(dest));
                    }
                    Ok(())
                }
            }
            #[derive(Clone, PartialEq, Eq, Hash, HeapSizeOf)]
            pub struct T(pub Vec<FontFamily>);
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![FontFamily::FamilyName(Atom::from_slice("serif"))])
        }
        /// <family-name>#
        /// <family-name> = <string> | [ <ident>+ ]
        /// TODO: <generic-family>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.parse_comma_separated(parse_one_family).map(SpecifiedValue)
        }
        pub fn parse_one_family(input: &mut Parser) -> Result<FontFamily, ()> {
            if let Ok(value) = input.try(|input| input.expect_string()) {
                return Ok(FontFamily::FamilyName(Atom::from_slice(&value)))
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
            Ok(FontFamily::FamilyName(Atom::from_slice(&value)))
        }
    </%self:longhand>


    ${single_keyword("font-style", "normal italic oblique")}
    ${single_keyword("font-variant", "normal small-caps")}

    <%self:longhand name="font-weight">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, PartialEq, Eq, Copy)]
        pub enum SpecifiedValue {
            Bolder,
            Lighter,
            % for weight in range(100, 901, 100):
                Weight${weight},
            % endfor
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
            #[derive(PartialEq, Eq, Copy, Clone, Hash, Deserialize, Serialize, HeapSizeOf)]
            pub enum T {
                % for weight in range(100, 901, 100):
                    Weight${weight} = ${weight},
                % endfor
            }
            impl fmt::Debug for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        % for weight in range(100, 901, 100):
                            &T::Weight${weight} => write!(f, "{}", ${weight}),
                        % endfor
                    }
                }
            }
            impl T {
                #[inline]
                pub fn is_bold(self) -> bool {
                    match self {
                        T::Weight900 | T::Weight800 |
                        T::Weight700 | T::Weight600 => true,
                        _ => false
                    }
                }
            }
        }
        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    % for weight in range(100, 901, 100):
                        computed_value::T::Weight${weight} => dest.write_str("${weight}"),
                    % endfor
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Weight400  // normal
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
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
        }
    </%self:longhand>

    <%self:longhand name="font-size">
        use cssparser::ToCss;
        use std::fmt;
        use util::geometry::Au;
        use values::computed::Context;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(pub specified::Length);  // Percentages are the same as em.
        pub mod computed_value {
            use util::geometry::Au;
            pub type T = Au;
        }
        const MEDIUM_PX: i32 = 16;
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(MEDIUM_PX)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                // We already computed this element's font size; no need to compute it again.
                return context.font_size
            }
        }
        /// <length> | <percentage> | <absolute-size> | <relative-size>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.try(specified::LengthOrPercentage::parse_non_negative)
            .map(|value| match value {
                specified::LengthOrPercentage::Length(value) => value,
                specified::LengthOrPercentage::Percentage(value) =>
                    specified::Length::FontRelative(specified::FontRelativeLength::Em(value))
            })
            .or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "xx-small" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX) * 3 / 5)),
                    "x-small" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX) * 3 / 4)),
                    "small" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX) * 8 / 9)),
                    "medium" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX))),
                    "large" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX) * 6 / 5)),
                    "x-large" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX) * 3 / 2)),
                    "xx-large" => Ok(specified::Length::Absolute(Au::from_px(MEDIUM_PX) * 2)),

                    // https://github.com/servo/servo/issues/3423#issuecomment-56321664
                    "smaller" => Ok(specified::Length::FontRelative(specified::FontRelativeLength::Em(0.85))),
                    "larger" => Ok(specified::Length::FontRelative(specified::FontRelativeLength::Em(1.2)))

                    _ => Err(())
                }
            })
            .map(SpecifiedValue)
        }
    </%self:longhand>

    ${single_keyword("font-stretch",
                     "normal ultra-condensed extra-condensed condensed semi-condensed semi-expanded \
                     expanded extra-expanded ultra-expanded")}

    // CSS 2.1, Section 16 - Text

    ${new_style_struct("InheritedText", is_inherited=True)}

    <%self:longhand name="text-align">
        pub use self::computed_value::T as SpecifiedValue;
        use values::computed::ComputedValueAsSpecified;
        impl ComputedValueAsSpecified for SpecifiedValue {}
        pub mod computed_value {
            macro_rules! define_text_align {
                ( $( $name: ident ( $string: expr ) => $discriminant: expr, )+ ) => {
                    define_css_keyword_enum! { T:
                        $(
                            $string => $name,
                        )+
                    }
                    impl T {
                        pub fn to_u32(self) -> u32 {
                            match self {
                                $(
                                    T::$name => $discriminant,
                                )+
                            }
                        }
                        pub fn from_u32(discriminant: u32) -> Option<T> {
                            match discriminant {
                                $(
                                    $discriminant => Some(T::$name),
                                )+
                                _ => None
                            }
                        }
                    }
                }
            }
            define_text_align! {
                start("start") => 0,
                end("end") => 1,
                left("left") => 2,
                right("right") => 3,
                center("center") => 4,
                justify("justify") => 5,
                servo_center("-servo-center") => 6,
            }
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::T::start
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser)
                     -> Result<SpecifiedValue, ()> {
            computed_value::T::parse(input)
        }
    </%self:longhand>

    <%self:longhand name="letter-spacing">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("normal"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
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
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),  // FIXME(SimonSapin) support percentages
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("normal"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
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

    ${single_keyword("unicode-bidi", "normal embed isolate bidi-override isolate-override plaintext")}

    <%self:longhand name="text-decoration" custom_cascade="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::ComputedValueAsSpecified;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        #[derive(PartialEq, Eq, Copy, Clone, Debug, HeapSizeOf)]
        pub struct SpecifiedValue {
            pub underline: bool,
            pub overline: bool,
            pub line_through: bool,
            // 'blink' is accepted in the parser but ignored.
            // Just not blinking the text is a conforming implementation per CSS 2.1.
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
            while let Ok(ident) = input.expect_ident() {
                match_ignore_ascii_case! { ident,
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

        fn cascade_property_custom(computed_value: &computed_value::T,
                                   _declaration: &PropertyDeclaration,
                                   style: &mut ComputedValues,
                                   _inherited_style: &ComputedValues,
                                   context: &computed::Context,
                                   _seen: &mut PropertyBitField,
                                   _cacheable: &mut bool) {
            Arc::make_unique(&mut style.inheritedtext)._servo_text_decorations_in_effect =
                longhands::_servo_text_decorations_in_effect::derive_from_text_decoration(
                    *computed_value,
                    &context);
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedText")}

    <%self:longhand name="-servo-text-decorations-in-effect"
                    derived_from="display text-decoration">
        use cssparser::{RGBA, ToCss};
        use std::fmt;

        use values::computed::ComputedValueAsSpecified;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
        pub struct SpecifiedValue {
            pub underline: Option<RGBA>,
            pub overline: Option<RGBA>,
            pub line_through: Option<RGBA>,
        }

        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                // Web compat doesn't matter here.
                Ok(())
            }
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

    ${single_keyword("border-collapse", "separate collapse")}

    ${single_keyword("empty-cells", "show hide")}

    ${single_keyword("caption-side", "top bottom")}

    <%self:longhand name="border-spacing">
        use values::computed::Context;

        use cssparser::ToCss;
        use std::fmt;
        use util::geometry::Au;

        pub mod computed_value {
            use util::geometry::Au;

            #[derive(Clone, Copy, Debug, PartialEq, RustcEncodable, HeapSizeOf)]
            pub struct T {
                pub horizontal: Au,
                pub vertical: Au,
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct SpecifiedValue {
            pub horizontal: specified::Length,
            pub vertical: specified::Length,
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                horizontal: Au(0),
                vertical: Au(0),
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                self.vertical.to_css(dest)
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                self.vertical.to_css(dest)
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                computed_value::T {
                    horizontal: self.horizontal.to_computed_value(context),
                    vertical: self.vertical.to_computed_value(context),
                }
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let mut lengths = [ None, None ];
            for i in 0..2 {
                match specified::Length::parse_non_negative(input) {
                    Err(()) => break,
                    Ok(length) => lengths[i] = Some(length),
                }
            }
            if input.next().is_ok() {
                return Err(())
            }
            match (lengths[0], lengths[1]) {
                (None, None) => Err(()),
                (Some(length), None) => {
                    Ok(SpecifiedValue {
                        horizontal: length,
                        vertical: length,
                    })
                }
                (Some(horizontal), Some(vertical)) => {
                    Ok(SpecifiedValue {
                        horizontal: horizontal,
                        vertical: vertical,
                    })
                }
                (None, Some(_)) => panic!("shouldn't happen"),
            }
        }
    </%self:longhand>

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
        pub use self::computed_value::T as SpecifiedValue;
        use values::computed::ComputedValueAsSpecified;

        impl ComputedValueAsSpecified for SpecifiedValue {}

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;
            use util::cursor::Cursor;

            #[derive(Clone, PartialEq, Eq, Copy, Debug, HeapSizeOf)]
            pub enum T {
                AutoCursor,
                SpecifiedCursor(Cursor),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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


    ${new_style_struct("Column", is_inherited=False)}

    <%self:longhand name="column-width" experimental="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Auto,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Auto => dest.write_str("auto"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("auto"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Auto => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(SpecifiedValue::Auto)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    <%self:longhand name="column-count" experimental="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Auto,
            Specified(u32),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Auto => dest.write_str("auto"),
                    SpecifiedValue::Specified(count) => write!(dest, "{}", count),
                }
            }
        }

        pub mod computed_value {
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<u32>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("auto"),
                    Some(count) => write!(dest, "{}", count),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Auto => computed_value::T(None),
                    SpecifiedValue::Specified(count) =>
                        computed_value::T(Some(count))
                }
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(SpecifiedValue::Auto)
            } else {
                let count = try!(input.expect_integer());
                // Zero is invalid
                if count <= 0 {
                    return Err(())
                }
                Ok(SpecifiedValue::Specified(count as u32))
            }
        }
    </%self:longhand>

    <%self:longhand name="column-gap" experimental="True">
        use cssparser::ToCss;
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Au>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("normal"),
                    Some(l) => l.to_css(dest),
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Normal => computed_value::T(None),
                    SpecifiedValue::Specified(l) =>
                        computed_value::T(Some(l.to_computed_value(context)))
                }
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

    // Box-shadow, etc.
    ${new_style_struct("Effects", is_inherited=False)}

    <%self:longhand name="opacity">
        use cssparser::ToCss;
        use std::fmt;
        use values::CSSFloat;
        use values::computed::Context;

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                self.0.to_css(dest)
            }
        }

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(pub CSSFloat);
        pub mod computed_value {
            use values::CSSFloat;
            pub type T = CSSFloat;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            1.0
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _context: &Context) -> computed_value::T {
                if self.0 < 0.0 {
                    0.0
                } else if self.0 > 1.0 {
                    1.0
                } else {
                    self.0
                }
            }
        }
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.expect_number().map(SpecifiedValue)
        }
    </%self:longhand>

    <%self:longhand name="box-shadow">
        use cssparser::{self, ToCss};
        use std::fmt;
        use values::computed::Context;

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(Vec<SpecifiedBoxShadow>);

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedBoxShadow {
            pub offset_x: specified::Length,
            pub offset_y: specified::Length,
            pub blur_radius: specified::Length,
            pub spread_radius: specified::Length,
            pub color: Option<specified::CSSColor>,
            pub inset: bool,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
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
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
            use std::fmt;
            use util::geometry::Au;
            use values::computed;

            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<BoxShadow>);

            #[derive(Clone, PartialEq, Copy, HeapSizeOf)]
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

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
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

        impl ToCss for computed_value::BoxShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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
                try!(dest.write_str(" "));
                try!(self.color.to_css(dest));
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue(Vec::new()))
            } else {
                input.parse_comma_separated(parse_one_box_shadow).map(SpecifiedValue)
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                computed_value::T(self.0.iter().map(|value| compute_one_box_shadow(value, context)).collect())
            }
        }

        pub fn compute_one_box_shadow(value: &SpecifiedBoxShadow, context: &computed::Context)
                                      -> computed_value::BoxShadow {
            computed_value::BoxShadow {
                offset_x: value.offset_x.to_computed_value(context),
                offset_y: value.offset_y.to_computed_value(context),
                blur_radius: value.blur_radius.to_computed_value(context),
                spread_radius: value.spread_radius.to_computed_value(context),
                color: value.color
                            .as_ref()
                            .map(|color| color.parsed)
                            .unwrap_or(cssparser::Color::CurrentColor),
                inset: value.inset,
            }
        }

        pub fn parse_one_box_shadow(input: &mut Parser) -> Result<SpecifiedBoxShadow, ()> {
            use util::geometry::Au;
            let mut lengths = [specified::Length::Absolute(Au(0)); 4];
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
        use std::fmt;

        // NB: `top` and `left` are 0 if `auto` per CSS 2.1 11.1.2.

        use values::computed::Context;

        pub mod computed_value {
            use util::geometry::Au;

            #[derive(Clone, PartialEq, Eq, Copy, Debug, HeapSizeOf)]
            pub struct ClipRect {
                pub top: Au,
                pub right: Option<Au>,
                pub bottom: Option<Au>,
                pub left: Au,
            }

            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<ClipRect>);
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => dest.write_str("auto"),
                    Some(rect) => {
                        try!(dest.write_str("rect("));
                        try!(rect.top.to_css(dest));
                        try!(dest.write_str(", "));
                        if let Some(right) = rect.right {
                            try!(right.to_css(dest));
                            try!(dest.write_str(", "));
                        } else {
                            try!(dest.write_str("auto, "));
                        }

                        if let Some(bottom) = rect.bottom {
                            try!(bottom.to_css(dest));
                            try!(dest.write_str(", "));
                        } else {
                            try!(dest.write_str("auto, "));
                        }

                        try!(rect.left.to_css(dest));
                        try!(dest.write_str(")"));
                        Ok(())
                    }
                }
            }
        }

        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct SpecifiedClipRect {
            pub top: specified::Length,
            pub right: Option<specified::Length>,
            pub bottom: Option<specified::Length>,
            pub left: specified::Length,
        }

        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct SpecifiedValue(Option<SpecifiedClipRect>);

        impl ToCss for SpecifiedClipRect {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(dest.write_str("rect("));

                try!(self.top.to_css(dest));
                try!(dest.write_str(", "));

                if let Some(right) = self.right {
                    try!(right.to_css(dest));
                    try!(dest.write_str(", "));
                } else {
                    try!(dest.write_str("auto, "));
                }

                if let Some(bottom) = self.bottom {
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

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                if let Some(ref rect) = self.0 {
                    rect.to_css(dest)
                } else {
                    dest.write_str("auto")
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                computed_value::T(self.0.map(|value| computed_value::ClipRect {
                    top: value.top.to_computed_value(context),
                    right: value.right.map(|right| right.to_computed_value(context)),
                    bottom: value.bottom.map(|bottom| bottom.to_computed_value(context)),
                    left: value.left.to_computed_value(context),
                }))
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use std::ascii::AsciiExt;
            use util::geometry::Au;
            use values::specified::Length;

            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                return Ok(SpecifiedValue(None))
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
                Ok(SpecifiedValue(Some(SpecifiedClipRect {
                    top: sides[0].unwrap_or(Length::Absolute(Au(0))),
                    right: sides[1],
                    bottom: sides[2],
                    left: sides[3].unwrap_or(Length::Absolute(Au(0))),
                })))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    <%self:longhand name="text-shadow">
        use cssparser::{self, ToCss};
        use std::fmt;

        use values::computed::Context;

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(Vec<SpecifiedTextShadow>);

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedTextShadow {
            pub offset_x: specified::Length,
            pub offset_y: specified::Length,
            pub blur_radius: specified::Length,
            pub color: Option<specified::CSSColor>,
        }

        impl fmt::Debug for SpecifiedTextShadow {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let _ = write!(f,
                               "{:?} {:?} {:?}",
                               self.offset_x,
                               self.offset_y,
                               self.blur_radius);
                if let Some(ref color) = self.color {
                    let _ = write!(f, "{:?}", color);
                }
                Ok(())
            }
        }

        pub mod computed_value {
            use cssparser::Color;
            use util::geometry::Au;

            #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
            pub struct T(pub Vec<TextShadow>);

            #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
            pub struct TextShadow {
                pub offset_x: Au,
                pub offset_y: Au,
                pub blur_radius: Au,
                pub color: Color,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
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

        impl ToCss for computed_value::TextShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.blur_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.color.to_css(dest));
                Ok(())
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
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

        impl ToCss for SpecifiedTextShadow {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.blur_radius.to_css(dest));

                if let Some(ref color) = self.color {
                    try!(dest.write_str(" "));
                    try!(color.to_css(dest));
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue(Vec::new()))
            } else {
                input.parse_comma_separated(parse_one_text_shadow).map(|shadows| {
                    SpecifiedValue(shadows)
                })
            }
        }

        fn parse_one_text_shadow(input: &mut Parser) -> Result<SpecifiedTextShadow,()> {
            use util::geometry::Au;
            let mut lengths = [specified::Length::Absolute(Au(0)); 3];
            let mut lengths_parsed = false;
            let mut color = None;

            loop {
                if !lengths_parsed {
                    if let Ok(value) = input.try(specified::Length::parse) {
                        lengths[0] = value;
                        let mut length_parsed_count = 1;
                        while length_parsed_count < 3 {
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

            Ok(SpecifiedTextShadow {
                offset_x: lengths[0],
                offset_y: lengths[1],
                blur_radius: lengths[2],
                color: color,
            })
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            fn to_computed_value(&self, context: &computed::Context) -> computed_value::T {
                computed_value::T(self.0.iter().map(|value| {
                    computed_value::TextShadow {
                        offset_x: value.offset_x.to_computed_value(context),
                        offset_y: value.offset_y.to_computed_value(context),
                        blur_radius: value.blur_radius.to_computed_value(context),
                        color: value.color
                                    .as_ref()
                                    .map(|color| color.parsed)
                                    .unwrap_or(cssparser::Color::CurrentColor),
                    }
                }).collect())
            }
        }
    </%self:longhand>

    <%self:longhand name="filter">
        //pub use self::computed_value::T as SpecifiedValue;
        use cssparser::ToCss;
        use std::fmt;
        use values::CSSFloat;
        use values::specified::{Angle, Length};

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedValue(Vec<SpecifiedFilter>);

        // TODO(pcwalton): `drop-shadow`
        #[derive(Clone, PartialEq, Debug)]
        pub enum SpecifiedFilter {
            Blur(Length),
            Brightness(CSSFloat),
            Contrast(CSSFloat),
            Grayscale(CSSFloat),
            HueRotate(Angle),
            Invert(CSSFloat),
            Opacity(CSSFloat),
            Saturate(CSSFloat),
            Sepia(CSSFloat),
        }

        pub mod computed_value {
            use util::geometry::Au;
            use values::CSSFloat;
            use values::specified::{Angle};

            #[derive(Clone, PartialEq, Debug, HeapSizeOf, Deserialize, Serialize)]
            pub enum Filter {
                Blur(Au),
                Brightness(CSSFloat),
                Contrast(CSSFloat),
                Grayscale(CSSFloat),
                HueRotate(Angle),
                Invert(CSSFloat),
                Opacity(CSSFloat),
                Saturate(CSSFloat),
                Sepia(CSSFloat),
            }

            #[derive(Clone, PartialEq, Debug, HeapSizeOf, Deserialize, Serialize)]
            pub struct T { pub filters: Vec<Filter> }

            impl T {
                /// Creates a new filter pipeline.
                #[inline]
                pub fn new(filters: Vec<Filter>) -> T {
                    T
                    {
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

                    for filter in &self.filters {
                        if let Filter::Opacity(ref opacity_value) = *filter {
                            opacity *= *opacity_value
                        }
                    }
                    opacity
                }
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
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

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut iter = self.0.iter();
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

        impl ToCss for computed_value::Filter {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    computed_value::Filter::Blur(value) => {
                        try!(dest.write_str("blur("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    computed_value::Filter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                    computed_value::Filter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                    computed_value::Filter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                    computed_value::Filter::HueRotate(value) => {
                        try!(dest.write_str("hue-rotate("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    computed_value::Filter::Invert(value) => try!(write!(dest, "invert({})", value)),
                    computed_value::Filter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                    computed_value::Filter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                    computed_value::Filter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                }
                Ok(())
            }
        }

        impl ToCss for SpecifiedFilter {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedFilter::Blur(value) => {
                        try!(dest.write_str("blur("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    SpecifiedFilter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                    SpecifiedFilter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                    SpecifiedFilter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                    SpecifiedFilter::HueRotate(value) => {
                        try!(dest.write_str("hue-rotate("));
                        try!(value.to_css(dest));
                        try!(dest.write_str(")"));
                    }
                    SpecifiedFilter::Invert(value) => try!(write!(dest, "invert({})", value)),
                    SpecifiedFilter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                    SpecifiedFilter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                    SpecifiedFilter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::new(Vec::new())
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            let mut filters = Vec::new();
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(filters))
            }
            loop {
                if let Ok(function_name) = input.try(|input| input.expect_function()) {
                    filters.push(try!(input.parse_nested_block(|input| {
                        match_ignore_ascii_case! { function_name,
                            "blur" => specified::Length::parse_non_negative(input).map(SpecifiedFilter::Blur),
                            "brightness" => parse_factor(input).map(SpecifiedFilter::Brightness),
                            "contrast" => parse_factor(input).map(SpecifiedFilter::Contrast),
                            "grayscale" => parse_factor(input).map(SpecifiedFilter::Grayscale),
                            "hue-rotate" => Angle::parse(input).map(SpecifiedFilter::HueRotate),
                            "invert" => parse_factor(input).map(SpecifiedFilter::Invert),
                            "opacity" => parse_factor(input).map(SpecifiedFilter::Opacity),
                            "saturate" => parse_factor(input).map(SpecifiedFilter::Saturate),
                            "sepia" => parse_factor(input).map(SpecifiedFilter::Sepia)
                            _ => Err(())
                        }
                    })));
                } else if filters.is_empty() {
                    return Err(())
                } else {
                    return Ok(SpecifiedValue(filters))
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

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            fn to_computed_value(&self, context: &computed::Context) -> computed_value::T {
                computed_value::T{ filters: self.0.iter().map(|value| {
                    match value {
                        &SpecifiedFilter::Blur(factor) =>
                            computed_value::Filter::Blur(factor.to_computed_value(context)),
                        &SpecifiedFilter::Brightness(factor) => computed_value::Filter::Brightness(factor),
                        &SpecifiedFilter::Contrast(factor) => computed_value::Filter::Contrast(factor),
                        &SpecifiedFilter::Grayscale(factor) => computed_value::Filter::Grayscale(factor),
                        &SpecifiedFilter::HueRotate(factor) => computed_value::Filter::HueRotate(factor),
                        &SpecifiedFilter::Invert(factor) => computed_value::Filter::Invert(factor),
                        &SpecifiedFilter::Opacity(factor) => computed_value::Filter::Opacity(factor),
                        &SpecifiedFilter::Saturate(factor) => computed_value::Filter::Saturate(factor),
                        &SpecifiedFilter::Sepia(factor) => computed_value::Filter::Sepia(factor),
                    }
                }).collect() }
            }
        }
    </%self:longhand>

    <%self:longhand name="transform">
        use values::CSSFloat;
        use values::computed::Context;

        use cssparser::ToCss;
        use std::fmt;
        use util::geometry::Au;

        pub mod computed_value {
            use values::CSSFloat;
            use values::computed;

            #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
            pub struct ComputedMatrix {
                pub m11: CSSFloat, pub m12: CSSFloat, pub m13: CSSFloat, pub m14: CSSFloat,
                pub m21: CSSFloat, pub m22: CSSFloat, pub m23: CSSFloat, pub m24: CSSFloat,
                pub m31: CSSFloat, pub m32: CSSFloat, pub m33: CSSFloat, pub m34: CSSFloat,
                pub m41: CSSFloat, pub m42: CSSFloat, pub m43: CSSFloat, pub m44: CSSFloat,
            }

            impl ComputedMatrix {
                pub fn identity() -> ComputedMatrix {
                    ComputedMatrix {
                        m11: 1.0, m12: 0.0, m13: 0.0, m14: 0.0,
                        m21: 0.0, m22: 1.0, m23: 0.0, m24: 0.0,
                        m31: 0.0, m32: 0.0, m33: 1.0, m34: 0.0,
                        m41: 0.0, m42: 0.0, m43: 0.0, m44: 1.0
                    }
                }
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum ComputedOperation {
                Matrix(ComputedMatrix),
                Skew(CSSFloat, CSSFloat),
                Translate(computed::LengthOrPercentage,
                          computed::LengthOrPercentage,
                          computed::Length),
                Scale(CSSFloat, CSSFloat, CSSFloat),
                Rotate(CSSFloat, CSSFloat, CSSFloat, computed::Angle),
                Perspective(computed::Length),
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub struct T(pub Option<Vec<ComputedOperation>>);
        }

        pub use self::computed_value::ComputedMatrix as SpecifiedMatrix;

        fn parse_two_lengths_or_percentages(input: &mut Parser)
                                            -> Result<(specified::LengthOrPercentage,
                                                       specified::LengthOrPercentage),()> {
            let first = try!(specified::LengthOrPercentage::parse(input));
            let second = input.try(|input| {
                try!(input.expect_comma());
                specified::LengthOrPercentage::parse(input)
            }).unwrap_or(specified::LengthOrPercentage::zero());
            Ok((first, second))
        }

        fn parse_two_floats(input: &mut Parser) -> Result<(CSSFloat,CSSFloat),()> {
            let first = try!(input.expect_number());
            let second = input.try(|input| {
                try!(input.expect_comma());
                input.expect_number()
            }).unwrap_or(first);
            Ok((first, second))
        }

        #[derive(Copy, Clone, Debug, PartialEq)]
        enum TranslateKind {
            Translate,
            TranslateX,
            TranslateY,
            TranslateZ,
            Translate3D,
        }

        #[derive(Clone, Debug, PartialEq)]
        enum SpecifiedOperation {
            Matrix(SpecifiedMatrix),
            Skew(CSSFloat, CSSFloat),
            Translate(TranslateKind,
                      specified::LengthOrPercentage,
                      specified::LengthOrPercentage,
                      specified::Length),
            Scale(CSSFloat, CSSFloat, CSSFloat),
            Rotate(CSSFloat, CSSFloat, CSSFloat, specified::Angle),
            Perspective(specified::Length),
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, _: &mut W) -> fmt::Result where W: fmt::Write {
                // TODO(pcwalton)
                Ok(())
            }
        }

        impl ToCss for SpecifiedOperation {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    // todo(gw): implement serialization for transform
                    // types other than translate.
                    SpecifiedOperation::Matrix(_m) => {
                        Ok(())
                    }
                    SpecifiedOperation::Skew(_sx, _sy) => {
                        Ok(())
                    }
                    SpecifiedOperation::Translate(kind, tx, ty, tz) => {
                        match kind {
                            TranslateKind::Translate => {
                                try!(dest.write_str("translate("));
                                try!(tx.to_css(dest));
                                try!(dest.write_str(", "));
                                try!(ty.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::TranslateX => {
                                try!(dest.write_str("translateX("));
                                try!(tx.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::TranslateY => {
                                try!(dest.write_str("translateY("));
                                try!(ty.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::TranslateZ => {
                                try!(dest.write_str("translateZ("));
                                try!(tz.to_css(dest));
                                dest.write_str(")")
                            }
                            TranslateKind::Translate3D => {
                                try!(dest.write_str("translate3d("));
                                try!(tx.to_css(dest));
                                try!(dest.write_str(", "));
                                try!(ty.to_css(dest));
                                try!(dest.write_str(", "));
                                try!(tz.to_css(dest));
                                dest.write_str(")")
                            }
                        }
                    }
                    SpecifiedOperation::Scale(_sx, _sy, _sz) => {
                        Ok(())
                    }
                    SpecifiedOperation::Rotate(_ax, _ay, _az, _angle) => {
                        Ok(())
                    }
                    SpecifiedOperation::Perspective(_p) => {
                        Ok(())
                    }
                }
            }
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct SpecifiedValue(Vec<SpecifiedOperation>);

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                let mut first = true;
                for operation in &self.0 {
                    if !first {
                        try!(dest.write_str(" "));
                    }
                    first = false;
                    try!(operation.to_css(dest))
                }
                Ok(())
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue(Vec::new()))
            }

            let mut result = Vec::new();
            loop {
                let name = match input.expect_function() {
                    Ok(name) => name,
                    Err(_) => break,
                };
                match_ignore_ascii_case! {
                    name,
                    "matrix" => {
                        try!(input.parse_nested_block(|input| {
                            let values = try!(input.parse_comma_separated(|input| {
                                input.expect_number()
                            }));
                            if values.len() != 6 {
                                return Err(())
                            }
                            result.push(SpecifiedOperation::Matrix(
                                    SpecifiedMatrix {
                                        m11: values[0], m12: values[1], m13: 0.0, m14: 0.0,
                                        m21: values[2], m22: values[3], m23: 0.0, m24: 0.0,
                                        m31:       0.0, m32:       0.0, m33: 1.0, m34: 0.0,
                                        m41: values[4], m42: values[5], m43: 0.0, m44: 1.0
                                    }));
                            Ok(())
                        }))
                    },
                    "matrix3d" => {
                        try!(input.parse_nested_block(|input| {
                            let values = try!(input.parse_comma_separated(|input| {
                                input.expect_number()
                            }));
                            if values.len() != 16 {
                                return Err(())
                            }
                            result.push(SpecifiedOperation::Matrix(
                                    SpecifiedMatrix {
                                        m11: values[ 0], m12: values[ 1], m13: values[ 2], m14: values[ 3],
                                        m21: values[ 4], m22: values[ 5], m23: values[ 6], m24: values[ 7],
                                        m31: values[ 8], m32: values[ 9], m33: values[10], m34: values[11],
                                        m41: values[12], m42: values[13], m43: values[14], m44: values[15]
                                    }));
                            Ok(())
                        }))
                    },
                    "translate" => {
                        try!(input.parse_nested_block(|input| {
                            let (tx, ty) = try!(parse_two_lengths_or_percentages(input));
                            result.push(SpecifiedOperation::Translate(TranslateKind::Translate,
                                                                      tx,
                                                                      ty,
                                                                      specified::Length::Absolute(Au(0))));
                            Ok(())
                        }))
                    },
                    "translatex" => {
                        try!(input.parse_nested_block(|input| {
                            let tx = try!(specified::LengthOrPercentage::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::TranslateX,
                                tx,
                                specified::LengthOrPercentage::zero(),
                                specified::Length::Absolute(Au(0))));
                            Ok(())
                        }))
                    },
                    "translatey" => {
                        try!(input.parse_nested_block(|input| {
                            let ty = try!(specified::LengthOrPercentage::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::TranslateY,
                                specified::LengthOrPercentage::zero(),
                                ty,
                                specified::Length::Absolute(Au(0))));
                            Ok(())
                        }))
                    },
                    "translatez" => {
                        try!(input.parse_nested_block(|input| {
                            let tz = try!(specified::Length::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::TranslateZ,
                                specified::LengthOrPercentage::zero(),
                                specified::LengthOrPercentage::zero(),
                                tz));
                            Ok(())
                        }))
                    },
                    "translate3d" => {
                        try!(input.parse_nested_block(|input| {
                            let tx = try!(specified::LengthOrPercentage::parse(input));
                            try!(input.expect_comma());
                            let ty = try!(specified::LengthOrPercentage::parse(input));
                            try!(input.expect_comma());
                            let tz = try!(specified::Length::parse(input));
                            result.push(SpecifiedOperation::Translate(
                                TranslateKind::Translate3D,
                                tx,
                                ty,
                                tz));
                            Ok(())
                        }))

                    },
                    "scale" => {
                        try!(input.parse_nested_block(|input| {
                            let (sx, sy) = try!(parse_two_floats(input));
                            result.push(SpecifiedOperation::Scale(sx, sy, 1.0));
                            Ok(())
                        }))
                    },
                    "scalex" => {
                        try!(input.parse_nested_block(|input| {
                            let sx = try!(input.expect_number());
                            result.push(SpecifiedOperation::Scale(sx, 1.0, 1.0));
                            Ok(())
                        }))
                    },
                    "scaley" => {
                        try!(input.parse_nested_block(|input| {
                            let sy = try!(input.expect_number());
                            result.push(SpecifiedOperation::Scale(1.0, sy, 1.0));
                            Ok(())
                        }))
                    },
                    "scalez" => {
                        try!(input.parse_nested_block(|input| {
                            let sz = try!(input.expect_number());
                            result.push(SpecifiedOperation::Scale(1.0, 1.0, sz));
                            Ok(())
                        }))
                    },
                    "scale3d" => {
                        try!(input.parse_nested_block(|input| {
                            let sx = try!(input.expect_number());
                            try!(input.expect_comma());
                            let sy = try!(input.expect_number());
                            try!(input.expect_comma());
                            let sz = try!(input.expect_number());
                            result.push(SpecifiedOperation::Scale(sx, sy, sz));
                            Ok(())
                        }))
                    },
                    "rotate" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(0.0, 0.0, 1.0, theta));
                            Ok(())
                        }))
                    },
                    "rotatex" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(1.0, 0.0, 0.0, theta));
                            Ok(())
                        }))
                    },
                    "rotatey" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(0.0, 1.0, 0.0, theta));
                            Ok(())
                        }))
                    },
                    "rotatez" => {
                        try!(input.parse_nested_block(|input| {
                            let theta = try!(specified::Angle::parse(input));
                            result.push(SpecifiedOperation::Rotate(0.0, 0.0, 1.0, theta));
                            Ok(())
                        }))
                    },
                    "rotate3d" => {
                        try!(input.parse_nested_block(|input| {
                            let ax = try!(input.expect_number());
                            try!(input.expect_comma());
                            let ay = try!(input.expect_number());
                            try!(input.expect_comma());
                            let az = try!(input.expect_number());
                            try!(input.expect_comma());
                            let theta = try!(specified::Angle::parse(input));
                            // TODO(gw): Check the axis can be normalized!!
                            result.push(SpecifiedOperation::Rotate(ax, ay, az, theta));
                            Ok(())
                        }))
                    },
                    "skew" => {
                        try!(input.parse_nested_block(|input| {
                            let (sx, sy) = try!(parse_two_floats(input));
                            result.push(SpecifiedOperation::Skew(sx, sy));
                            Ok(())
                        }))
                    },
                    "skewx" => {
                        try!(input.parse_nested_block(|input| {
                            let sx = try!(input.expect_number());
                            result.push(SpecifiedOperation::Skew(sx, 1.0));
                            Ok(())
                        }))
                    },
                    "skewy" => {
                        try!(input.parse_nested_block(|input| {
                            let sy = try!(input.expect_number());
                            result.push(SpecifiedOperation::Skew(1.0, sy));
                            Ok(())
                        }))
                    },
                    "perspective" => {
                        try!(input.parse_nested_block(|input| {
                            let d = try!(specified::Length::parse(input));
                            result.push(SpecifiedOperation::Perspective(d));
                            Ok(())
                        }))
                    }
                    _ => return Err(())
                }
            }

            if !result.is_empty() {
                Ok(SpecifiedValue(result))
            } else {
                Err(())
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                if self.0.is_empty() {
                    return computed_value::T(None)
                }

                let mut result = vec!();
                for operation in &self.0 {
                    match *operation {
                        SpecifiedOperation::Matrix(ref matrix) => {
                            result.push(computed_value::ComputedOperation::Matrix(*matrix));
                        }
                        SpecifiedOperation::Translate(_, ref tx, ref ty, ref tz) => {
                            result.push(computed_value::ComputedOperation::Translate(tx.to_computed_value(context),
                                                                                     ty.to_computed_value(context),
                                                                                     tz.to_computed_value(context)));
                        }
                        SpecifiedOperation::Scale(sx, sy, sz) => {
                            result.push(computed_value::ComputedOperation::Scale(sx, sy, sz));
                        }
                        SpecifiedOperation::Rotate(ax, ay, az, theta) => {
                            result.push(computed_value::ComputedOperation::Rotate(ax, ay, az, theta));
                        }
                        SpecifiedOperation::Skew(sx, sy) => {
                            result.push(computed_value::ComputedOperation::Skew(sx, sy));
                        }
                        SpecifiedOperation::Perspective(d) => {
                            result.push(computed_value::ComputedOperation::Perspective(d.to_computed_value(context)));
                        }
                    };
                }

                computed_value::T(Some(result))
            }
        }
    </%self:longhand>

    pub struct OriginParseResult {
        horizontal: Option<specified::LengthOrPercentage>,
        vertical: Option<specified::LengthOrPercentage>,
        depth: Option<specified::Length>
    }

    pub fn parse_origin(_: &ParserContext, input: &mut Parser) -> Result<OriginParseResult,()> {
        let (mut horizontal, mut vertical, mut depth) = (None, None, None);
        loop {
            if let Err(_) = input.try(|input| {
                let token = try!(input.expect_ident());
                match_ignore_ascii_case! {
                    token,
                    "left" => {
                        if horizontal.is_none() {
                            horizontal = Some(specified::LengthOrPercentage::Percentage(0.0))
                        } else {
                            return Err(())
                        }
                    },
                    "center" => {
                        if horizontal.is_none() {
                            horizontal = Some(specified::LengthOrPercentage::Percentage(0.5))
                        } else if vertical.is_none() {
                            vertical = Some(specified::LengthOrPercentage::Percentage(0.5))
                        } else {
                            return Err(())
                        }
                    },
                    "right" => {
                        if horizontal.is_none() {
                            horizontal = Some(specified::LengthOrPercentage::Percentage(1.0))
                        } else {
                            return Err(())
                        }
                    },
                    "top" => {
                        if vertical.is_none() {
                            vertical = Some(specified::LengthOrPercentage::Percentage(0.0))
                        } else {
                            return Err(())
                        }
                    },
                    "bottom" => {
                        if vertical.is_none() {
                            vertical = Some(specified::LengthOrPercentage::Percentage(1.0))
                        } else {
                            return Err(())
                        }
                    }
                    _ => return Err(())
                }
                Ok(())
            }) {
                match specified::LengthOrPercentage::parse(input) {
                    Ok(value) => {
                        if horizontal.is_none() {
                            horizontal = Some(value);
                        } else if vertical.is_none() {
                            vertical = Some(value);
                        } else if let specified::LengthOrPercentage::Length(length) = value {
                            depth = Some(length);
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }

        if horizontal.is_some() || vertical.is_some() {
            Ok(OriginParseResult {
                horizontal: horizontal,
                vertical: vertical,
                depth: depth,
            })
        } else {
            Err(())
        }
    }

    ${single_keyword("backface-visibility", "visible hidden")}

    ${single_keyword("transform-style", "auto flat preserve-3d")}

    <%self:longhand name="transform-origin">
        use values::computed::Context;
        use values::specified::{Length, LengthOrPercentage};

        use cssparser::ToCss;
        use std::fmt;
        use util::geometry::Au;

        pub mod computed_value {
            use values::computed::{Length, LengthOrPercentage};

            #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
            pub struct T {
                pub horizontal: LengthOrPercentage,
                pub vertical: LengthOrPercentage,
                pub depth: Length,
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct SpecifiedValue {
            horizontal: LengthOrPercentage,
            vertical: LengthOrPercentage,
            depth: Length,
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.vertical.to_css(dest));
                try!(dest.write_str(" "));
                self.depth.to_css(dest)
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.vertical.to_css(dest));
                try!(dest.write_str(" "));
                self.depth.to_css(dest)
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                horizontal: computed::LengthOrPercentage::Percentage(0.5),
                vertical: computed::LengthOrPercentage::Percentage(0.5),
                depth: Au(0),
            }
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let result = try!(super::parse_origin(context, input));
            Ok(SpecifiedValue {
                horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(0.5)),
                vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(0.5)),
                depth: result.depth.unwrap_or(Length::Absolute(Au(0))),
            })
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                computed_value::T {
                    horizontal: self.horizontal.to_computed_value(context),
                    vertical: self.vertical.to_computed_value(context),
                    depth: self.depth.to_computed_value(context),
                }
            }
        }
    </%self:longhand>

    ${predefined_type("perspective",
                      "LengthOrNone",
                      "computed::LengthOrNone::None")}

    <%self:longhand name="perspective-origin">
        use values::computed::Context;
        use values::specified::LengthOrPercentage;

        use cssparser::ToCss;
        use std::fmt;

        pub mod computed_value {
            use values::computed::LengthOrPercentage;

            #[derive(Clone, Copy, Debug, PartialEq, HeapSizeOf)]
            pub struct T {
                pub horizontal: LengthOrPercentage,
                pub vertical: LengthOrPercentage,
            }
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                self.vertical.to_css(dest)
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct SpecifiedValue {
            horizontal: LengthOrPercentage,
            vertical: LengthOrPercentage,
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                try!(self.horizontal.to_css(dest));
                try!(dest.write_str(" "));
                self.vertical.to_css(dest)
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                horizontal: computed::LengthOrPercentage::Percentage(0.5),
                vertical: computed::LengthOrPercentage::Percentage(0.5),
            }
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            let result = try!(super::parse_origin(context, input));
            match result.depth {
                Some(_) => Err(()),
                None => Ok(SpecifiedValue {
                    horizontal: result.horizontal.unwrap_or(LengthOrPercentage::Percentage(0.5)),
                    vertical: result.vertical.unwrap_or(LengthOrPercentage::Percentage(0.5)),
                })
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                computed_value::T {
                    horizontal: self.horizontal.to_computed_value(context),
                    vertical: self.vertical.to_computed_value(context),
                }
            }
        }
    </%self:longhand>

    ${single_keyword("mix-blend-mode",
                     """normal multiply screen overlay darken lighten color-dodge
                        color-burn hard-light soft-light difference exclusion hue
                        saturation color luminosity""")}

    <%self:longhand name="image-rendering">
        use values::computed::Context;

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf, Deserialize, Serialize)]
            pub enum T {
                Auto,
                CrispEdges,
                Pixelated,
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        T::Auto => dest.write_str("auto"),
                        T::CrispEdges => dest.write_str("crisp-edges"),
                        T::Pixelated => dest.write_str("pixelated"),
                    }
                }
            }
        }

        pub type SpecifiedValue = computed_value::T;

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Auto
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            // According to to CSS-IMAGES-3, `optimizespeed` and `optimizequality` are synonyms for
            // `auto`.
            match_ignore_ascii_case! {
                try!(input.expect_ident()),
                "auto" => Ok(computed_value::T::Auto),
                "optimizespeed" => Ok(computed_value::T::Auto),
                "optimizequality" => Ok(computed_value::T::Auto),
                "crisp-edges" => Ok(computed_value::T::CrispEdges),
                "pixelated" => Ok(computed_value::T::Pixelated)
                _ => Err(())
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _: &Context) -> computed_value::T {
                *self
            }
        }
    </%self:longhand>

    ${new_style_struct("Animation", is_inherited=False)}

    // TODO(pcwalton): Multiple transitions.
    <%self:longhand name="transition-duration">
        use values::specified::Time;

        pub use self::computed_value::T as SpecifiedValue;
        pub use values::specified::Time as SingleSpecifiedValue;

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;
            use values::computed::{Context, ToComputedValue};

            pub use values::computed::Time as SingleComputedValue;

            #[derive(Clone, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<SingleComputedValue>);

            impl ToComputedValue for T {
                type ComputedValue = T;

                #[inline]
                fn to_computed_value(&self, _: &Context) -> T {
                    (*self).clone()
                }
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    if self.0.is_empty() {
                        return dest.write_str("none")
                    }
                    for (i, value) in self.0.iter().enumerate() {
                        if i != 0 {
                            try!(dest.write_str(", "))
                        }
                        try!(value.to_css(dest))
                    }
                    Ok(())
                }
            }
        }

        #[inline]
        pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
            Time::parse(input)
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![get_initial_single_value()])
        }

        #[inline]
        pub fn get_initial_single_value() -> Time {
            Time(0.0)
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
        }
    </%self:longhand>

    // TODO(pcwalton): Lots more timing functions.
    // TODO(pcwalton): Multiple transitions.
    <%self:longhand name="transition-timing-function">
        use self::computed_value::{StartEnd, TransitionTimingFunction};
        use values::computed::Context;

        use euclid::point::Point2D;

        pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
        pub use self::computed_value::T as SpecifiedValue;

        static EASE: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.25,
            y: 0.1,
        }, Point2D {
            x: 0.25,
            y: 1.0,
        });
        static LINEAR: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.0,
            y: 0.0,
        }, Point2D {
            x: 1.0,
            y: 1.0,
        });
        static EASE_IN: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.42,
            y: 0.0,
        }, Point2D {
            x: 1.0,
            y: 1.0,
        });
        static EASE_OUT: TransitionTimingFunction = TransitionTimingFunction::CubicBezier(Point2D {
            x: 0.0,
            y: 0.0,
        }, Point2D {
            x: 0.58,
            y: 1.0,
        });
        static EASE_IN_OUT: TransitionTimingFunction =
            TransitionTimingFunction::CubicBezier(Point2D {
                x: 0.42,
                y: 0.0,
            }, Point2D {
                x: 0.58,
                y: 1.0,
            });
        static STEP_START: TransitionTimingFunction =
            TransitionTimingFunction::Steps(1, StartEnd::Start);
        static STEP_END: TransitionTimingFunction =
            TransitionTimingFunction::Steps(1, StartEnd::End);

        pub mod computed_value {
            use cssparser::ToCss;
            use euclid::point::Point2D;
            use std::fmt;

            pub use self::TransitionTimingFunction as SingleComputedValue;

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum TransitionTimingFunction {
                CubicBezier(Point2D<f32>, Point2D<f32>),
                Steps(u32, StartEnd),
            }

            impl ToCss for TransitionTimingFunction {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        TransitionTimingFunction::CubicBezier(p1, p2) => {
                            try!(dest.write_str("cubic-bezier("));
                            try!(p1.x.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(p1.y.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(p2.x.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(p2.y.to_css(dest));
                            dest.write_str(")")
                        }
                        TransitionTimingFunction::Steps(steps, start_end) => {
                            try!(dest.write_str("steps("));
                            try!(steps.to_css(dest));
                            try!(dest.write_str(", "));
                            try!(start_end.to_css(dest));
                            dest.write_str(")")
                        }
                    }
                }
            }

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum StartEnd {
                Start,
                End,
            }

            impl ToCss for StartEnd {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        StartEnd::Start => dest.write_str("start"),
                        StartEnd::End => dest.write_str("end"),
                    }
                }
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<TransitionTimingFunction>);

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    if self.0.is_empty() {
                        return dest.write_str("none")
                    }
                    for (i, value) in self.0.iter().enumerate() {
                        if i != 0 {
                            try!(dest.write_str(", "))
                        }
                        try!(value.to_css(dest))
                    }
                    Ok(())
                }
            }
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _: &Context) -> computed_value::T {
                (*self).clone()
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(vec![get_initial_single_value()])
        }

        #[inline]
        pub fn get_initial_single_value() -> TransitionTimingFunction {
            EASE
        }

        pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
            if let Ok(function_name) = input.try(|input| input.expect_function()) {
                return match_ignore_ascii_case! {
                    function_name,
                    "cubic-bezier" => {
                        let (mut p1x, mut p1y, mut p2x, mut p2y) = (0.0, 0.0, 0.0, 0.0);
                        try!(input.parse_nested_block(|input| {
                            p1x = try!(input.expect_number());
                            try!(input.expect_comma());
                            p1y = try!(input.expect_number());
                            try!(input.expect_comma());
                            p2x = try!(input.expect_number());
                            try!(input.expect_comma());
                            p2y = try!(input.expect_number());
                            Ok(())
                        }));
                        let (p1, p2) = (Point2D::new(p1x, p1y), Point2D::new(p2x, p2y));
                        Ok(TransitionTimingFunction::CubicBezier(p1, p2))
                    },
                    "steps" => {
                        let (mut step_count, mut start_end) = (0, computed_value::StartEnd::Start);
                        try!(input.parse_nested_block(|input| {
                            step_count = try!(input.expect_integer());
                            try!(input.expect_comma());
                            start_end = try!(match_ignore_ascii_case! {
                                try!(input.expect_ident()),
                                "start" => Ok(computed_value::StartEnd::Start),
                                "end" => Ok(computed_value::StartEnd::End)
                                _ => Err(())
                            });
                            Ok(())
                        }));
                        Ok(TransitionTimingFunction::Steps(step_count as u32, start_end))
                    }
                    _ => Err(())
                }
            }
            match_ignore_ascii_case! {
                try!(input.expect_ident()),
                "ease" => Ok(EASE),
                "linear" => Ok(LINEAR),
                "ease-in" => Ok(EASE_IN),
                "ease-out" => Ok(EASE_OUT),
                "ease-in-out" => Ok(EASE_IN_OUT),
                "step-start" => Ok(STEP_START),
                "step-end" => Ok(STEP_END)
                _ => Err(())
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
        }
    </%self:longhand>

    // TODO(pcwalton): Lots more properties.
    <%self:longhand name="transition-property">
        use self::computed_value::TransitionProperty;
        use values::computed::Context;

        pub use self::computed_value::SingleComputedValue as SingleSpecifiedValue;
        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            use cssparser::ToCss;
            use std::fmt;

            pub use self::TransitionProperty as SingleComputedValue;

            #[derive(Copy, Clone, Debug, PartialEq, HeapSizeOf)]
            pub enum TransitionProperty {
                All,
                BackgroundColor,
                BackgroundPosition,
                BorderBottomColor,
                BorderBottomWidth,
                BorderLeftColor,
                BorderLeftWidth,
                BorderRightColor,
                BorderRightWidth,
                BorderSpacing,
                BorderTopColor,
                BorderTopWidth,
                Bottom,
                Color,
                Clip,
                FontSize,
                FontWeight,
                Height,
                Left,
                LetterSpacing,
                LineHeight,
                MarginBottom,
                MarginLeft,
                MarginRight,
                MarginTop,
                MaxHeight,
                MaxWidth,
                MinHeight,
                MinWidth,
                Opacity,
                OutlineColor,
                OutlineWidth,
                PaddingBottom,
                PaddingLeft,
                PaddingRight,
                PaddingTop,
                Right,
                TextIndent,
                TextShadow,
                Top,
                Transform,
                VerticalAlign,
                Visibility,
                Width,
                WordSpacing,
                ZIndex,
            }

            pub static ALL_TRANSITION_PROPERTIES: [TransitionProperty; 45] = [
                TransitionProperty::BackgroundColor,
                TransitionProperty::BackgroundPosition,
                TransitionProperty::BorderBottomColor,
                TransitionProperty::BorderBottomWidth,
                TransitionProperty::BorderLeftColor,
                TransitionProperty::BorderLeftWidth,
                TransitionProperty::BorderRightColor,
                TransitionProperty::BorderRightWidth,
                TransitionProperty::BorderSpacing,
                TransitionProperty::BorderTopColor,
                TransitionProperty::BorderTopWidth,
                TransitionProperty::Bottom,
                TransitionProperty::Color,
                TransitionProperty::Clip,
                TransitionProperty::FontSize,
                TransitionProperty::FontWeight,
                TransitionProperty::Height,
                TransitionProperty::Left,
                TransitionProperty::LetterSpacing,
                TransitionProperty::LineHeight,
                TransitionProperty::MarginBottom,
                TransitionProperty::MarginLeft,
                TransitionProperty::MarginRight,
                TransitionProperty::MarginTop,
                TransitionProperty::MaxHeight,
                TransitionProperty::MaxWidth,
                TransitionProperty::MinHeight,
                TransitionProperty::MinWidth,
                TransitionProperty::Opacity,
                TransitionProperty::OutlineColor,
                TransitionProperty::OutlineWidth,
                TransitionProperty::PaddingBottom,
                TransitionProperty::PaddingLeft,
                TransitionProperty::PaddingRight,
                TransitionProperty::PaddingTop,
                TransitionProperty::Right,
                TransitionProperty::TextIndent,
                TransitionProperty::TextShadow,
                TransitionProperty::Top,
                TransitionProperty::Transform,
                TransitionProperty::VerticalAlign,
                TransitionProperty::Visibility,
                TransitionProperty::Width,
                TransitionProperty::WordSpacing,
                TransitionProperty::ZIndex,
            ];

            impl ToCss for TransitionProperty {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    match *self {
                        TransitionProperty::All => dest.write_str("all"),
                        TransitionProperty::BackgroundColor => dest.write_str("background-color"),
                        TransitionProperty::BackgroundPosition => dest.write_str("background-position"),
                        TransitionProperty::BorderBottomColor => dest.write_str("border-bottom-color"),
                        TransitionProperty::BorderBottomWidth => dest.write_str("border-bottom-width"),
                        TransitionProperty::BorderLeftColor => dest.write_str("border-left-color"),
                        TransitionProperty::BorderLeftWidth => dest.write_str("border-left-width"),
                        TransitionProperty::BorderRightColor => dest.write_str("border-right-color"),
                        TransitionProperty::BorderRightWidth => dest.write_str("border-right-width"),
                        TransitionProperty::BorderSpacing => dest.write_str("border-spacing"),
                        TransitionProperty::BorderTopColor => dest.write_str("border-top-color"),
                        TransitionProperty::BorderTopWidth => dest.write_str("border-top-width"),
                        TransitionProperty::Bottom => dest.write_str("bottom"),
                        TransitionProperty::Color => dest.write_str("color"),
                        TransitionProperty::Clip => dest.write_str("clip"),
                        TransitionProperty::FontSize => dest.write_str("font-size"),
                        TransitionProperty::FontWeight => dest.write_str("font-weight"),
                        TransitionProperty::Height => dest.write_str("height"),
                        TransitionProperty::Left => dest.write_str("left"),
                        TransitionProperty::LetterSpacing => dest.write_str("letter-spacing"),
                        TransitionProperty::LineHeight => dest.write_str("line-height"),
                        TransitionProperty::MarginBottom => dest.write_str("margin-bottom"),
                        TransitionProperty::MarginLeft => dest.write_str("margin-left"),
                        TransitionProperty::MarginRight => dest.write_str("margin-right"),
                        TransitionProperty::MarginTop => dest.write_str("margin-top"),
                        TransitionProperty::MaxHeight => dest.write_str("max-height"),
                        TransitionProperty::MaxWidth => dest.write_str("max-width"),
                        TransitionProperty::MinHeight => dest.write_str("min-height"),
                        TransitionProperty::MinWidth => dest.write_str("min-width"),
                        TransitionProperty::Opacity => dest.write_str("opacity"),
                        TransitionProperty::OutlineColor => dest.write_str("outline-color"),
                        TransitionProperty::OutlineWidth => dest.write_str("outline-width"),
                        TransitionProperty::PaddingBottom => dest.write_str("padding-bottom"),
                        TransitionProperty::PaddingLeft => dest.write_str("padding-left"),
                        TransitionProperty::PaddingRight => dest.write_str("padding-right"),
                        TransitionProperty::PaddingTop => dest.write_str("padding-top"),
                        TransitionProperty::Right => dest.write_str("right"),
                        TransitionProperty::TextIndent => dest.write_str("text-indent"),
                        TransitionProperty::TextShadow => dest.write_str("text-shadow"),
                        TransitionProperty::Top => dest.write_str("top"),
                        TransitionProperty::Transform => dest.write_str("transform"),
                        TransitionProperty::VerticalAlign => dest.write_str("vertical-align"),
                        TransitionProperty::Visibility => dest.write_str("visibility"),
                        TransitionProperty::Width => dest.write_str("width"),
                        TransitionProperty::WordSpacing => dest.write_str("word-spacing"),
                        TransitionProperty::ZIndex => dest.write_str("z-index"),
                    }
                }
            }

            #[derive(Clone, Debug, PartialEq, HeapSizeOf)]
            pub struct T(pub Vec<SingleComputedValue>);

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                    if self.0.is_empty() {
                        return dest.write_str("none")
                    }
                    for (i, value) in self.0.iter().enumerate() {
                        if i != 0 {
                            try!(dest.write_str(", "))
                        }
                        try!(value.to_css(dest))
                    }
                    Ok(())
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T(Vec::new())
        }

        pub fn parse_one(input: &mut Parser) -> Result<SingleSpecifiedValue,()> {
            match_ignore_ascii_case! {
                try!(input.expect_ident()),
                "all" => Ok(TransitionProperty::All),
                "background-color" => Ok(TransitionProperty::BackgroundColor),
                "background-position" => Ok(TransitionProperty::BackgroundPosition),
                "border-bottom-color" => Ok(TransitionProperty::BorderBottomColor),
                "border-bottom-width" => Ok(TransitionProperty::BorderBottomWidth),
                "border-left-color" => Ok(TransitionProperty::BorderLeftColor),
                "border-left-width" => Ok(TransitionProperty::BorderLeftWidth),
                "border-right-color" => Ok(TransitionProperty::BorderRightColor),
                "border-right-width" => Ok(TransitionProperty::BorderRightWidth),
                "border-spacing" => Ok(TransitionProperty::BorderSpacing),
                "border-top-color" => Ok(TransitionProperty::BorderTopColor),
                "border-top-width" => Ok(TransitionProperty::BorderTopWidth),
                "bottom" => Ok(TransitionProperty::Bottom),
                "color" => Ok(TransitionProperty::Color),
                "clip" => Ok(TransitionProperty::Clip),
                "font-size" => Ok(TransitionProperty::FontSize),
                "font-weight" => Ok(TransitionProperty::FontWeight),
                "height" => Ok(TransitionProperty::Height),
                "left" => Ok(TransitionProperty::Left),
                "letter-spacing" => Ok(TransitionProperty::LetterSpacing),
                "line-height" => Ok(TransitionProperty::LineHeight),
                "margin-bottom" => Ok(TransitionProperty::MarginBottom),
                "margin-left" => Ok(TransitionProperty::MarginLeft),
                "margin-right" => Ok(TransitionProperty::MarginRight),
                "margin-top" => Ok(TransitionProperty::MarginTop),
                "max-height" => Ok(TransitionProperty::MaxHeight),
                "max-width" => Ok(TransitionProperty::MaxWidth),
                "min-height" => Ok(TransitionProperty::MinHeight),
                "min-width" => Ok(TransitionProperty::MinWidth),
                "opacity" => Ok(TransitionProperty::Opacity),
                "outline-color" => Ok(TransitionProperty::OutlineColor),
                "outline-width" => Ok(TransitionProperty::OutlineWidth),
                "padding-bottom" => Ok(TransitionProperty::PaddingBottom),
                "padding-left" => Ok(TransitionProperty::PaddingLeft),
                "padding-right" => Ok(TransitionProperty::PaddingRight),
                "padding-top" => Ok(TransitionProperty::PaddingTop),
                "right" => Ok(TransitionProperty::Right),
                "text-indent" => Ok(TransitionProperty::TextIndent),
                "text-shadow" => Ok(TransitionProperty::TextShadow),
                "top" => Ok(TransitionProperty::Top),
                "transform" => Ok(TransitionProperty::Transform),
                "vertical-align" => Ok(TransitionProperty::VerticalAlign),
                "visibility" => Ok(TransitionProperty::Visibility),
                "width" => Ok(TransitionProperty::Width),
                "word-spacing" => Ok(TransitionProperty::WordSpacing),
                "z-index" => Ok(TransitionProperty::ZIndex)
                _ => Err(())
            }
        }

        pub fn parse(_: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue,()> {
            Ok(SpecifiedValue(try!(input.parse_comma_separated(parse_one))))
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, _: &Context) -> computed_value::T {
                (*self).clone()
            }
        }
    </%self:longhand>

    <%self:longhand name="transition-delay">
        pub use properties::longhands::transition_duration::{SingleSpecifiedValue, SpecifiedValue};
        pub use properties::longhands::transition_duration::{computed_value};
        pub use properties::longhands::transition_duration::{get_initial_single_value};
        pub use properties::longhands::transition_duration::{get_initial_value, parse, parse_one};
    </%self:longhand>

    // CSS Flexible Box Layout Module Level 1
    // http://www.w3.org/TR/css3-flexbox/

    ${new_style_struct("Flex", is_inherited=False)}

    // Flex container properties
    ${single_keyword("flex-direction", "row row-reverse column column-reverse", experimental=True)}
}


pub mod shorthands {
    use cssparser::Parser;
    use parser::ParserContext;
    use values::specified;

    <%def name="shorthand(name, sub_properties, experimental=False)">
    <%
        shorthand = Shorthand(name, sub_properties.split(), experimental=experimental)
        SHORTHANDS.append(shorthand)
    %>
        pub mod ${shorthand.ident} {
            use cssparser::Parser;
            use parser::ParserContext;
            use properties::longhands;

            pub struct Longhands {
                % for sub_property in shorthand.sub_properties:
                    pub ${sub_property.ident}:
                        Option<longhands::${sub_property.ident}::SpecifiedValue>,
                % endfor
            }

            #[allow(unused_variables)]
            pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
                ${caller.body()}
            }
        }
    </%def>

    fn parse_four_sides<F, T>(input: &mut Parser, parse_one: F) -> Result<(T, T, T, T), ()>
    where F: Fn(&mut Parser) -> Result<T, ()>, F: Copy, T: Clone {
        // zero or more than four values is invalid.
        // one value sets them all
        // two values set (top, bottom) and (left, right)
        // three values set top, (left, right) and bottom
        // four values set them in order
        let top = try!(parse_one(input));
        let right;
        let bottom;
        let left;
        match input.try(parse_one) {
            Err(()) => {
                right = top.clone();
                bottom = top.clone();
                left = top.clone();
            }
            Ok(value) => {
                right = value;
                match input.try(parse_one) {
                    Err(()) => {
                        bottom = top.clone();
                        left = right.clone();
                    }
                    Ok(value) => {
                        bottom = value;
                        match input.try(parse_one) {
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
        Ok((top, right, bottom, left))
    }

    <%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
        <%self:shorthand name="${name}" sub_properties="${
                ' '.join(sub_property_pattern % side
                         for side in ['top', 'right', 'bottom', 'left'])}">
            use super::parse_four_sides;
            use values::specified;
            let _unused = context;
            let (top, right, bottom, left) = try!(parse_four_sides(input, ${parser_function}));
            Ok(Longhands {
                % for side in ["top", "right", "bottom", "left"]:
                    ${to_rust_ident(sub_property_pattern % side)}: Some(${side}),
                % endfor
            })
        </%self:shorthand>
    </%def>

    // TODO: other background-* properties
    <%self:shorthand name="background"
                     sub_properties="background-color background-position background-repeat background-attachment
                                     background-image background-size background-origin background-clip">
        use properties::longhands::{background_color, background_position, background_repeat, background_attachment};
        use properties::longhands::{background_image, background_size, background_origin, background_clip};

        let mut color = None;
        let mut image = None;
        let mut position = None;
        let mut repeat = None;
        let mut size = None;
        let mut attachment = None;
        let mut any = false;
        let mut origin = None;
        let mut clip = None;

        loop {
            if position.is_none() {
                if let Ok(value) = input.try(|input| background_position::parse(context, input)) {
                    position = Some(value);
                    any = true;

                    // Parse background size, if applicable.
                    size = input.try(|input| {
                        try!(input.expect_delim('/'));
                        background_size::parse(context, input)
                    }).ok();

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
            if origin.is_none() {
                if let Ok(value) = input.try(|input| background_origin::parse(context, input)) {
                    origin = Some(value);
                    any = true;
                    continue
                }
            }
            if clip.is_none() {
                if let Ok(value) = input.try(|input| background_clip::parse(context, input)) {
                    clip = Some(value);
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
                background_size: size,
                background_origin: origin,
                background_clip: clip,
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
    <%self:shorthand name="border-width" sub_properties="${
            ' '.join('border-%s-width' % side
                     for side in ['top', 'right', 'bottom', 'left'])}">
        use super::parse_four_sides;
        use values::specified;
        let _unused = context;
        let (top, right, bottom, left) = try!(parse_four_sides(input, specified::parse_border_width));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                ${to_rust_ident('border-%s-width' % side)}:
                    Some(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue(${side})),
            % endfor
        })
    </%self:shorthand>


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
                border_${side}_color: color,
                border_${side}_style: style,
                border_${side}_width:
                    width.map(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue),
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
                border_${side}_color: color.clone(),
                border_${side}_style: style,
                border_${side}_width:
                    width.map(longhands::${to_rust_ident('border-%s-width' % side)}::SpecifiedValue),
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
            let mut values = [LengthOrPercentage::Length(Length::Absolute(Au(0))); 4];
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
        use properties::longhands::outline_width;
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
                if let Ok(value) = input.try(|input| outline_width::parse(context, input)) {
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
            font_family: Some(font_family::SpecifiedValue(family))
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

    <%self:shorthand name="columns" sub_properties="column-count column-width" experimental="True">
        use properties::longhands::{column_count, column_width};
        let mut column_count = None;
        let mut column_width = None;
        let mut autos = 0;

        loop {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                // Leave the options to None, 'auto' is the initial value.
                autos += 1;
                continue
            }

            if column_count.is_none() {
                if let Ok(value) = input.try(|input| column_count::parse(context, input)) {
                    column_count = Some(value);
                    continue
                }
            }

            if column_width.is_none() {
                if let Ok(value) = input.try(|input| column_width::parse(context, input)) {
                    column_width = Some(value);
                    continue
                }
            }

            break
        }

        let values = autos + column_count.iter().len() + column_width.iter().len();
        if values == 0 || values > 2 {
            Err(())
        } else {
            Ok(Longhands {
                column_count: column_count,
                column_width: column_width,
            })
        }
    </%self:shorthand>

    <%self:shorthand name="overflow" sub_properties="overflow-x overflow-y">
        use properties::longhands::{overflow_x, overflow_y};

        let overflow = try!(overflow_x::parse(context, input));
        Ok(Longhands {
            overflow_x: Some(overflow),
            overflow_y: Some(overflow_y::SpecifiedValue(overflow)),
        })
    </%self:shorthand>

    <%self:shorthand name="transition"
                     sub_properties="transition-property transition-duration transition-timing-function
                                     transition-delay">
        use properties::longhands::{transition_delay, transition_duration, transition_property};
        use properties::longhands::{transition_timing_function};

        struct SingleTransition {
            transition_property: transition_property::SingleSpecifiedValue,
            transition_duration: transition_duration::SingleSpecifiedValue,
            transition_timing_function: transition_timing_function::SingleSpecifiedValue,
            transition_delay: transition_delay::SingleSpecifiedValue,
        }

        fn parse_one_transition(input: &mut Parser) -> Result<SingleTransition,()> {
            let (mut property, mut duration) = (None, None);
            let (mut timing_function, mut delay) = (None, None);
            loop {
                if property.is_none() {
                    if let Ok(value) = input.try(|input| transition_property::parse_one(input)) {
                        property = Some(value);
                        continue
                    }
                }

                if duration.is_none() {
                    if let Ok(value) = input.try(|input| transition_duration::parse_one(input)) {
                        duration = Some(value);
                        continue
                    }
                }

                if timing_function.is_none() {
                    if let Ok(value) = input.try(|input| {
                        transition_timing_function::parse_one(input)
                    }) {
                        timing_function = Some(value);
                        continue
                    }
                }

                if delay.is_none() {
                    if let Ok(value) = input.try(|input| transition_delay::parse_one(input)) {
                        delay = Some(value);
                        continue;
                    }
                }

                break
            }

            if let Some(property) = property {
                Ok(SingleTransition {
                    transition_property: property,
                    transition_duration:
                        duration.unwrap_or(transition_duration::get_initial_single_value()),
                    transition_timing_function:
                        timing_function.unwrap_or(
                            transition_timing_function::get_initial_single_value()),
                    transition_delay:
                        delay.unwrap_or(transition_delay::get_initial_single_value()),
                })
            } else {
                Err(())
            }
        }

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(Longhands {
                transition_property: None,
                transition_duration: None,
                transition_timing_function: None,
                transition_delay: None,
            })
        }

        let results = try!(input.parse_comma_separated(parse_one_transition));
        let (mut properties, mut durations) = (Vec::new(), Vec::new());
        let (mut timing_functions, mut delays) = (Vec::new(), Vec::new());
        for result in results {
            properties.push(result.transition_property);
            durations.push(result.transition_duration);
            timing_functions.push(result.transition_timing_function);
            delays.push(result.transition_delay);
        }

        Ok(Longhands {
            transition_property: Some(transition_property::SpecifiedValue(properties)),
            transition_duration: Some(transition_duration::SpecifiedValue(durations)),
            transition_timing_function:
                Some(transition_timing_function::SpecifiedValue(timing_functions)),
            transition_delay: Some(transition_delay::SpecifiedValue(delays)),
        })
    </%self:shorthand>
}


// TODO(SimonSapin): Convert this to a syntax extension rather than a Mako template.
// Maybe submit for inclusion in libstd?
mod property_bit_field {

    pub struct PropertyBitField {
        storage: [u32; (${len(LONGHANDS)} - 1 + 32) / 32]
    }

    impl PropertyBitField {
        #[inline]
        pub fn new() -> PropertyBitField {
            PropertyBitField { storage: [0; (${len(LONGHANDS)} - 1 + 32) / 32] }
        }

        #[inline]
        fn get(&self, bit: usize) -> bool {
            (self.storage[bit / 32] & (1 << (bit % 32))) != 0
        }
        #[inline]
        fn set(&mut self, bit: usize) {
            self.storage[bit / 32] |= 1 << (bit % 32)
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
#[derive(Debug, PartialEq, HeapSizeOf)]
pub struct PropertyDeclarationBlock {
    #[ignore_heap_size_of = "#7038"]
    pub important: Arc<Vec<PropertyDeclaration>>,
    #[ignore_heap_size_of = "#7038"]
    pub normal: Arc<Vec<PropertyDeclaration>>,
}


pub fn parse_style_attribute(input: &str, base_url: &Url) -> PropertyDeclarationBlock {
    let context = ParserContext::new(Origin::Author, base_url);
    parse_property_declaration_list(&context, &mut Parser::new(input))
}

pub fn parse_one_declaration(name: &str, input: &str, base_url: &Url)
                             -> Result<Vec<PropertyDeclaration>, ()> {
    let context = ParserContext::new(Origin::Author, base_url);
    let mut results = vec![];
    match PropertyDeclaration::parse(name, &context, &mut Parser::new(input), &mut results) {
        PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => Ok(results),
        _ => Err(())
    }
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
                    important_declarations.extend(results);
                } else {
                    normal_declarations.extend(results);
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


#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

#[derive(PartialEq, Clone)]
pub enum PropertyDeclaration {
    % for property in LONGHANDS:
        ${property.camel_case}(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
}


#[derive(Eq, PartialEq, Copy, Clone)]
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
                    % if shorthand.experimental:
                        if !::util::opts::experimental_enabled() {
                            return PropertyDeclarationParseResult::ExperimentalProperty
                        }
                    % endif
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
        #[derive(PartialEq, Clone, HeapSizeOf)]
        pub struct ${style_struct.name} {
            % for longhand in style_struct.longhands:
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
            % if style_struct.name == "Font":
                pub hash: u64,
            % endif
        }
    % endfor
}

#[derive(Clone, HeapSizeOf)]
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
        if self.writing_mode.is_vertical() {
            box_style.height
        } else {
            box_style.width
        }
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
    pub fn is_multicol(&self) -> bool {
        let style = self.get_column();
        style.column_count.0.is_some() || style.column_width.0.is_some()
    }

    #[inline]
    pub fn get_font_arc(&self) -> Arc<style_structs::Font> {
        self.font.clone()
    }

    // http://dev.w3.org/csswg/css-transforms/#grouping-property-values
    pub fn get_used_transform_style(&self) -> computed_values::transform_style::T {
        use computed_values::mix_blend_mode;
        use computed_values::transform_style;

        let effects = self.get_effects();

        // TODO(gw): Add clip-path, isolation, mask-image, mask-border-source when supported.
        if effects.opacity < 1.0 ||
           !effects.filter.is_empty() ||
           effects.clip.0.is_some() {
           effects.mix_blend_mode != mix_blend_mode::T::normal ||
            return transform_style::T::flat;
        }

        if effects.transform_style == transform_style::T::auto {
            if effects.transform.0.is_some() {
                return transform_style::T::flat;
            }
            if effects.perspective != computed::LengthOrNone::None {
                return transform_style::T::flat;
            }
        }

        // Return the computed value if not overridden by the above exceptions
        effects.transform_style
    }

    % for style_struct in STYLE_STRUCTS:
    #[inline]
    pub fn get_${style_struct.name.lower()}
            <'a>(&'a self) -> &'a style_structs::${style_struct.name} {
        &*self.${style_struct.ident}
    }
    #[inline]
    pub fn mutate_${style_struct.name.lower()}
            <'a>(&'a mut self) -> &'a mut style_structs::${style_struct.name} {
        &mut *Arc::make_unique(&mut self.${style_struct.ident})
    }
    % endfor

    pub fn computed_value_to_string(&self, name: &str) -> Option<String> {
        match name {
            % for style_struct in STYLE_STRUCTS:
                % for longhand in style_struct.longhands:
                "${longhand.name}" => Some(self.${style_struct.ident}.${longhand.ident}.to_css_string()),
                % endfor
            % endfor
            _ => None
        }
    }
}


/// Return a WritingMode bitflags from the relevant CSS properties.
pub fn get_writing_mode(inheritedbox_style: &style_structs::InheritedBox) -> WritingMode {
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
    pub static ref INITIAL_VALUES: ComputedValues = ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: Arc::new(style_structs::${style_struct.name} {
                % for longhand in style_struct.longhands:
                    ${longhand.ident}: longhands::${longhand.ident}::get_initial_value(),
                % endfor
                % if style_struct.name == "Font":
                    hash: 0,
                % endif
            }),
        % endfor
        shareable: true,
        writing_mode: WritingMode::empty(),
        root_font_size: longhands::font_size::get_initial_value(),
    };
}


/// Fast path for the function below. Only computes new inherited styles.
#[allow(unused_mut)]
fn cascade_with_cached_declarations(
        applicable_declarations: &[DeclarationBlock<Vec<PropertyDeclaration>>],
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
                            PropertyDeclaration::${property.camel_case}(ref
                                    ${'_' if not style_struct.inherited else ''}declared_value)
                                    => {
                                % if style_struct.inherited:
                                    if seen.get_${property.ident}() {
                                        continue
                                    }
                                    seen.set_${property.ident}();
                                    let computed_value = match *declared_value {
                                        DeclaredValue::SpecifiedValue(ref specified_value)
                                        => specified_value.to_computed_value(context),
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
                                    Arc::make_unique(&mut style_${style_struct.ident})
                                        .${property.ident} = computed_value;
                                % endif

                                % if property.name in DERIVED_LONGHANDS:
                                    % if not style_struct.inherited:
                                        // Use the cached value.
                                        let computed_value = style_${style_struct.ident}
                                            .${property.ident}.clone();
                                    % endif
                                    % for derived in DERIVED_LONGHANDS[property.name]:
                                        Arc::make_unique(&mut style_${derived.style_struct.ident})
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

    if seen.get_font_style() || seen.get_font_weight() || seen.get_font_stretch() ||
            seen.get_font_family() {
        compute_font_hash(&mut *Arc::make_unique(&mut style_font))
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

type CascadePropertyFn = extern "Rust" fn(declaration: &PropertyDeclaration,
                                          style: &mut ComputedValues,
                                          inherited_style: &ComputedValues,
                                          context: &computed::Context,
                                          seen: &mut PropertyBitField,
                                          cacheable: &mut bool);

// This is a thread-local rather than a lazy static to avoid atomic operations when cascading
// properties.
thread_local!(static CASCADE_PROPERTY: Vec<Option<CascadePropertyFn>> = {
    let mut result: Vec<Option<CascadePropertyFn>> = Vec::new();
    % for style_struct in STYLE_STRUCTS:
        % for property in style_struct.longhands:
            let discriminant;
            unsafe {
                let variant = PropertyDeclaration::${property.camel_case}(intrinsics::uninit());
                discriminant = intrinsics::discriminant_value(&variant) as usize;
                mem::forget(variant);
            }
            while result.len() < discriminant + 1 {
                result.push(None)
            }
            result[discriminant] = Some(longhands::${property.ident}::cascade_property);
        % endfor
    % endfor
    result
});

/// Performs the CSS cascade, computing new styles for an element from its parent style and
/// optionally a cached related style. The arguments are:
///
///   * `viewport_size`: The size of the initial viewport.
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
pub fn cascade(viewport_size: Size2D<Au>,
               applicable_declarations: &[DeclarationBlock<Vec<PropertyDeclaration>>],
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
            viewport_size: viewport_size,
            inherited_font_weight: inherited_font_style.font_weight,
            inherited_font_size: inherited_font_style.font_size,
            inherited_text_decorations_in_effect:
                inherited_style.get_inheritedtext()._servo_text_decorations_in_effect,
            // To be overridden by applicable declarations:
            font_size: inherited_font_style.font_size,
            root_font_size: inherited_style.root_font_size,
            display: longhands::display::get_initial_value(),
            color: inherited_style.get_color().color,
            text_decoration: longhands::text_decoration::get_initial_value(),
            overflow_x: longhands::overflow_x::get_initial_value(),
            overflow_y: longhands::overflow_y::get_initial_value(),
            positioned: false,
            floated: false,
            border_top_present: false,
            border_right_present: false,
            border_bottom_present: false,
            border_left_present: false,
            outline_style_present: false,
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
    for sub_list in applicable_declarations {
        // Declarations are stored in reverse source order, we want them in forward order here.
        for declaration in sub_list.declarations.iter().rev() {
            match *declaration {
                PropertyDeclaration::FontSize(ref value) => {
                    context.font_size = match *value {
                        DeclaredValue::SpecifiedValue(ref specified_value) => {
                            match specified_value.0 {
                                Length::FontRelative(value) => {
                                    value.to_computed_value(context.inherited_font_size,
                                                            context.root_font_size)
                                }
                                Length::ServoCharacterWidth(value) => {
                                    value.to_computed_value(context.inherited_font_size)
                                }
                                _ => specified_value.0.to_computed_value(&context)
                            }
                        }
                        DeclaredValue::Initial => longhands::font_size::get_initial_value(),
                        DeclaredValue::Inherit => context.inherited_font_size,
                    }
                }
                PropertyDeclaration::Color(ref value) => {
                    context.color = match *value {
                        DeclaredValue::SpecifiedValue(ref specified_value) => {
                            specified_value.parsed
                        }
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
                PropertyDeclaration::OverflowX(ref value) => {
                    context.overflow_x = get_specified!(get_box, overflow_x, value);
                }
                PropertyDeclaration::OverflowY(ref value) => {
                    context.overflow_y = get_specified!(get_box, overflow_y, value);
                }
                PropertyDeclaration::Float(ref value) => {
                    context.floated = get_specified!(get_box, float, value)
                                      != longhands::float::SpecifiedValue::none;
                }
                PropertyDeclaration::TextDecoration(ref value) => {
                    context.text_decoration = get_specified!(get_text, text_decoration, value);
                }
                PropertyDeclaration::OutlineStyle(ref value) => {
                    context.outline_style_present =
                        match get_specified!(get_outline, outline_style, value) {
                            BorderStyle::none => false,
                            _ => true,
                        };
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
    let mut style = ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}:
                % if style_struct.inherited:
                    inherited_style
                % else:
                    initial_values
                % endif
                .${style_struct.ident}.clone(),
        % endfor
        shareable: false,
        writing_mode: WritingMode::empty(),
        root_font_size: context.root_font_size,
    };
    let mut cacheable = true;
    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order, we want them in decreasing
    // order here.
    //
    // We could (and used to) use a pattern match here, but that bloats this function to over 100K
    // of compiled code! To improve i-cache behavior, we outline the individual functions and use
    // virtual dispatch instead.
    CASCADE_PROPERTY.with(|cascade_property| {
        for sub_list in applicable_declarations.iter().rev() {
            // Declarations are already stored in reverse order.
            for declaration in sub_list.declarations.iter() {
                let discriminant = unsafe {
                    intrinsics::discriminant_value(declaration) as usize
                };
                (cascade_property[discriminant].unwrap())(declaration,
                                                          &mut style,
                                                          inherited_style,
                                                          &context,
                                                          &mut seen,
                                                          &mut cacheable);
            }
        }
    });

    // The initial value of border-*-width may be changed at computed value time.
    {
        let border = Arc::make_unique(&mut style.border);
        % for side in ["top", "right", "bottom", "left"]:
            // Like calling to_computed_value, which wouldn't type check.
            if !context.border_${side}_present {
                border.border_${side}_width = Au(0);
            }
        % endfor
    }

    // The initial value of display may be changed at computed value time.
    if !seen.get_display() {
        let box_ = Arc::make_unique(&mut style.box_);
        box_.display = box_.display.to_computed_value(&context);
    }

    // The initial value of outline width may be changed at computed value time.
    if !context.outline_style_present {
        let outline = Arc::make_unique(&mut style.outline);
        outline.outline_width = Au(0);
    }

    if is_root_element {
        context.root_font_size = context.font_size;
    }

    if seen.get_font_style() || seen.get_font_weight() || seen.get_font_stretch() ||
            seen.get_font_family() {
        compute_font_hash(&mut *Arc::make_unique(&mut style.font))
    }

    (ComputedValues {
        writing_mode: get_writing_mode(&*style.inheritedbox),
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: style.${style_struct.ident},
        % endfor
        shareable: shareable,
        root_font_size: context.root_font_size,
    }, cacheable)
}

/// Alters the given style to accommodate replaced content. This is called in flow construction. It
/// handles cases like `<div style="position: absolute">foo bar baz</div>` (in which `foo`, `bar`,
/// and `baz` must not be absolutely-positioned) and cases like `<sup>Foo</sup>` (in which the
/// `vertical-align: top` style of `sup` must not propagate down into `Foo`).
///
/// FIXME(#5625, pcwalton): It would probably be cleaner and faster to do this in the cascade.
#[inline]
pub fn modify_style_for_replaced_content(style: &mut Arc<ComputedValues>) {
    // Reset `position` to handle cases like `<div style="position: absolute">foo bar baz</div>`.
    if style.box_.display != longhands::display::computed_value::T::inline {
        let mut style = Arc::make_unique(style);
        Arc::make_unique(&mut style.box_).display = longhands::display::computed_value::T::inline;
        Arc::make_unique(&mut style.box_).position =
            longhands::position::computed_value::T::static_;
    }

    // Reset `vertical-align` to handle cases like `<sup>foo</sup>`.
    if style.box_.vertical_align != longhands::vertical_align::computed_value::T::baseline {
        let mut style = Arc::make_unique(style);
        Arc::make_unique(&mut style.box_).vertical_align =
            longhands::vertical_align::computed_value::T::baseline
    }

    // Reset margins.
    if style.margin.margin_top != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_left != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_bottom != computed::LengthOrPercentageOrAuto::Length(Au(0)) ||
            style.margin.margin_right != computed::LengthOrPercentageOrAuto::Length(Au(0)) {
        let mut style = Arc::make_unique(style);
        let margin = Arc::make_unique(&mut style.margin);
        margin.margin_top = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_left = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_bottom = computed::LengthOrPercentageOrAuto::Length(Au(0));
        margin.margin_right = computed::LengthOrPercentageOrAuto::Length(Au(0));
    }
}

/// Adjusts borders, padding, and margins as appropriate to account for a fragment's status as the
/// first or last fragment within the range of an element.
///
/// Specifically, this function sets border/padding/margin widths to zero on the sides for which
/// the fragment is not outermost.
#[inline]
pub fn modify_style_for_inline_sides(style: &mut Arc<ComputedValues>,
                                     is_first_fragment_of_element: bool,
                                     is_last_fragment_of_element: bool) {
    fn modify_side(style: &mut Arc<ComputedValues>, side: PhysicalSide) {
        let mut style = Arc::make_unique(style);
        let border = Arc::make_unique(&mut style.border);
        match side {
            PhysicalSide::Left => {
                border.border_left_width = Au(0);
                border.border_left_style = BorderStyle::none;
                Arc::make_unique(&mut style.padding).padding_left =
                    computed::LengthOrPercentage::Length(Au(0));
                Arc::make_unique(&mut style.margin).margin_left =
                    computed::LengthOrPercentageOrAuto::Length(Au(0))
            }
            PhysicalSide::Right => {
                border.border_right_width = Au(0);
                border.border_right_style = BorderStyle::none;
                Arc::make_unique(&mut style.padding).padding_right =
                    computed::LengthOrPercentage::Length(Au(0));
                Arc::make_unique(&mut style.margin).margin_right =
                    computed::LengthOrPercentageOrAuto::Length(Au(0))
            }
            PhysicalSide::Bottom => {
                border.border_bottom_width = Au(0);
                border.border_bottom_style = BorderStyle::none;
                Arc::make_unique(&mut style.padding).padding_bottom =
                    computed::LengthOrPercentage::Length(Au(0));
                Arc::make_unique(&mut style.margin).margin_bottom =
                    computed::LengthOrPercentageOrAuto::Length(Au(0))
            }
            PhysicalSide::Top => {
                border.border_top_width = Au(0);
                border.border_top_style = BorderStyle::none;
                Arc::make_unique(&mut style.padding).padding_top =
                    computed::LengthOrPercentage::Length(Au(0));
                Arc::make_unique(&mut style.margin).margin_top =
                    computed::LengthOrPercentageOrAuto::Length(Au(0))
            }
        }
    }

    if !is_first_fragment_of_element {
        let side = style.writing_mode.inline_start_physical_side();
        modify_side(style, side)
    }

    if !is_last_fragment_of_element {
        let side = style.writing_mode.inline_end_physical_side();
        modify_side(style, side)
    }
}

/// Adjusts the display and position properties as appropriate for an anonymous table object.
#[inline]
pub fn modify_style_for_anonymous_table_object(
        style: &mut Arc<ComputedValues>,
        new_display_value: longhands::display::computed_value::T) {
    let mut style = Arc::make_unique(style);
    let box_style = Arc::make_unique(&mut style.box_);
    box_style.display = new_display_value;
    box_style.position = longhands::position::computed_value::T::static_;
}

/// Adjusts the `position` property as necessary for the outer fragment wrapper of an inline-block.
#[inline]
pub fn modify_style_for_outer_inline_block_fragment(style: &mut Arc<ComputedValues>) {
    let mut style = Arc::make_unique(style);
    let box_style = Arc::make_unique(&mut style.box_);
    box_style.position = longhands::position::computed_value::T::static_
}

/// Adjusts the `position` property as necessary to account for text.
///
/// Text is never directly relatively positioned; it's always contained within an element that is
/// itself relatively positioned.
#[inline]
pub fn modify_style_for_text(style: &mut Arc<ComputedValues>) {
    if style.box_.position == longhands::position::computed_value::T::relative {
        // We leave the `position` property set to `relative` so that we'll still establish a
        // containing block if needed. But we reset all position offsets to `auto`.
        let mut style = Arc::make_unique(style);
        let mut position_offsets = Arc::make_unique(&mut style.positionoffsets);
        position_offsets.top = computed::LengthOrPercentageOrAuto::Auto;
        position_offsets.right = computed::LengthOrPercentageOrAuto::Auto;
        position_offsets.bottom = computed::LengthOrPercentageOrAuto::Auto;
        position_offsets.left = computed::LengthOrPercentageOrAuto::Auto;
    }
}

/// Adjusts the `margin` property as necessary to account for the text of an `input` element.
///
/// Margins apply to the `input` element itself, so including them in the text will cause them to
/// be double-counted.
pub fn modify_style_for_input_text(style: &mut Arc<ComputedValues>) {
    let mut style = Arc::make_unique(style);
    let margin_style = Arc::make_unique(&mut style.margin);
    margin_style.margin_top = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_right = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_bottom = computed::LengthOrPercentageOrAuto::Length(Au(0));
    margin_style.margin_left = computed::LengthOrPercentageOrAuto::Length(Au(0));
}

/// Adjusts the `clip` property so that an inline absolute hypothetical fragment doesn't clip its
/// children.
pub fn modify_style_for_inline_absolute_hypothetical_fragment(style: &mut Arc<ComputedValues>) {
    if style.get_effects().clip.0.is_some() {
        let mut style = Arc::make_unique(style);
        let effects_style = Arc::make_unique(&mut style.effects);
        effects_style.clip.0 = None
    }
}

pub fn is_supported_property(property: &str) -> bool {
    match_ignore_ascii_case! { property,
        % for property in SHORTHANDS + LONGHANDS[:-1]:
            "${property.name}" => true,
        % endfor
        "${LONGHANDS[-1].name}" => true
        _ => false
    }
}

#[macro_export]
macro_rules! css_properties_accessors {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in SHORTHANDS + LONGHANDS:
                % if property.derived_from is None:
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

// Extra space here because < seems to be removed by Mako when immediately followed by &.
//                                                         ↓
pub fn longhands_from_shorthand(shorthand: &str) -> Option< &'static [&'static str]> {
    % for property in SHORTHANDS:
        static ${property.ident.upper()}: &'static [&'static str] = &[
            % for sub in property.sub_properties:
                "${sub.name}",
            % endfor
        ];
    % endfor
    match_ignore_ascii_case!{ shorthand,
        % for property in SHORTHANDS[:-1]:
            "${property.name}" => Some(${property.ident.upper()}),
        % endfor
        % for property in SHORTHANDS[-1:]:
            "${property.name}" => Some(${property.ident.upper()})
        % endfor
        _ => None
    }
}

/// Corresponds to the fields in `gfx::font_template::FontTemplateDescriptor`.
fn compute_font_hash(font: &mut style_structs::Font) {
    let mut hasher: FnvHasher = Default::default();
    hasher.write_u16(font.font_weight as u16);
    font.font_stretch.hash(&mut hasher);
    font.font_family.hash(&mut hasher);
    font.hash = hasher.finish()
}
