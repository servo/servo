/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

#[allow(non_camel_case_types)];

pub use servo_util::url::parse_url;
use sync::Arc;
pub use extra::url::Url;
use servo_util::cowarc::CowArc;

pub use cssparser::*;
pub use cssparser::ast::*;

use errors::{ErrorLoggerIterator, log_css_error};
pub use parsing_utils::*;
pub use self::common_types::*;
use selector_matching::MatchedProperty;

use serialize::{Encodable, Encoder};

pub mod common_types;

<%!

def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super"]:  # Rust keywords
        name += "_"
    return name

class Longhand(object):
    def __init__(self, name):
        self.name = name
        self.ident = to_rust_ident(name)
        self.style_struct = THIS_STYLE_STRUCT

class Shorthand(object):
    def __init__(self, name, sub_properties):
        self.name = name
        self.ident = to_rust_ident(name)
        self.sub_properties = [LONGHANDS_BY_NAME[s] for s in sub_properties]

class StyleStruct(object):
    def __init__(self, name, inherited):
        self.name = name
        self.longhands = []
        self.inherited = inherited

STYLE_STRUCTS = []
THIS_STYLE_STRUCT = None
LONGHANDS = []
LONGHANDS_BY_NAME = {}
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

    <%def name="raw_longhand(name, no_super=False)">
    <%
        property = Longhand(name)
        THIS_STYLE_STRUCT.longhands.append(property)
        LONGHANDS.append(property)
        LONGHANDS_BY_NAME[name] = property
    %>
        pub mod ${property.ident} {
            % if not no_super:
                use super::*;
            % endif
            pub use self::computed_value::*;
            ${caller.body()}
            pub fn parse_declared(input: &[ComponentValue], base_url: &Url)
                               -> Option<DeclaredValue<SpecifiedValue>> {
                match CSSWideKeyword::parse(input) {
                    Some(Some(keyword)) => Some(CSSWideKeyword(keyword)),
                    Some(None) => Some(CSSWideKeyword(${
                        "Inherit" if THIS_STYLE_STRUCT.inherited else "Initial"})),
                    None => parse_specified(input, base_url),
                }
            }
        }
    </%def>

    <%def name="longhand(name, no_super=False)">
        <%self:raw_longhand name="${name}">
            ${caller.body()}
            pub fn parse_specified(input: &[ComponentValue], base_url: &Url)
                               -> Option<DeclaredValue<SpecifiedValue>> {
                parse(input, base_url).map(super::SpecifiedValue)
            }
        </%self:raw_longhand>
    </%def>

    <%def name="single_component_value(name)">
        <%self:longhand name="${name}">
            ${caller.body()}
            pub fn parse(input: &[ComponentValue], base_url: &Url) -> Option<SpecifiedValue> {
                one_component_value(input).and_then(|c| from_component_value(c, base_url))
            }
        </%self:longhand>
    </%def>

    <%def name="single_keyword_computed(name, values)">
        <%self:single_component_value name="${name}">
            ${caller.body()}
            pub mod computed_value {
                #[deriving(Eq, Clone, FromPrimitive)]
                pub enum T {
                    % for value in values.split():
                        ${to_rust_ident(value)},
                    % endfor
                }
            }
            pub type SpecifiedValue = computed_value::T;
            #[inline] pub fn get_initial_value() -> computed_value::T {
                ${to_rust_ident(values.split()[0])}
            }
            pub fn from_component_value(v: &ComponentValue, _base_url: &Url)
                                        -> Option<SpecifiedValue> {
                get_ident_lower(v).and_then(|keyword| {
                    match keyword.as_slice() {
                        % for value in values.split():
                            "${value}" => Some(${to_rust_ident(value)}),
                        % endfor
                        _ => None,
                    }
                })
            }
        </%self:single_component_value>
    </%def>

    <%def name="single_keyword(name, values)">
        <%self:single_keyword_computed name="${name}"
                                       values="${values}">
            // The computed value is the same as the specified value.
            pub use to_computed_value = super::computed_as_specified;
        </%self:single_keyword_computed>
    </%def>

    <%def name="predefined_type(name, type, initial_value, parse_method='parse')">
        <%self:single_component_value name="${name}">
            pub use to_computed_value = super::super::common_types::computed::compute_${type};
            pub type SpecifiedValue = specified::${type};
            pub mod computed_value {
                pub type T = super::super::computed::${type};
            }
            #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
            #[inline] pub fn from_component_value(v: &ComponentValue, _base_url: &Url)
                                                  -> Option<SpecifiedValue> {
                specified::${type}::${parse_method}(v)
            }
        </%self:single_component_value>
    </%def>


    // CSS 2.1, Section 8 - Box model

    ${new_style_struct("Margin", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("margin-" + side, "LengthOrPercentageOrAuto",
                          "computed::LPA_Length(Au(0))")}
    % endfor

    ${new_style_struct("Padding", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("padding-" + side, "LengthOrPercentage",
                          "computed::LP_Length(Au(0))",
                          "parse_non_negative")}
    % endfor

    ${new_style_struct("Border", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("border-%s-color" % side, "CSSColor", "CurrentColor")}
    % endfor

    //  double groove ridge insed outset
    ${single_keyword("border-top-style", values="none solid dotted dashed hidden")}

    % for side in ["right", "bottom", "left"]:
        <%self:longhand name="border-${side}-style", no_super="True">
            pub use super::border_top_style::{get_initial_value, parse, to_computed_value};
            pub type SpecifiedValue = super::border_top_style::SpecifiedValue;
            pub mod computed_value {
                pub type T = super::super::border_top_style::computed_value::T;
            }
        </%self:longhand>
    % endfor

    pub fn parse_border_width(component_value: &ComponentValue, _base_url: &Url)
                              -> Option<specified::Length> {
        match component_value {
            &Ident(ref value) => {
                // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
                unsafe {
                    let tmp_for_lifetime = value.to_ascii_nocheck().to_lower();
                    let value_lower = tmp_for_lifetime.as_str_ascii();
                    match value_lower.as_slice() {
                        "thin" => Some(specified::Length::from_px(1.)),
                        "medium" => Some(specified::Length::from_px(3.)),
                        "thick" => Some(specified::Length::from_px(5.)),
                        _ => None
                    }
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
            pub fn parse(input: &[ComponentValue], base_url: &Url) -> Option<SpecifiedValue> {
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

    ${new_style_struct("PositionOffsets", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type(side, "LengthOrPercentageOrAuto",
                          "computed::LPA_Auto")}
    % endfor

    // CSS 2.1, Section 9 - Visual formatting model

    ${new_style_struct("Box", is_inherited=False)}

    // TODO: don't parse values we don't support
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
                    inline_table => table,
                    inline | inline_block
                    | table_row_group | table_column | table_column_group
                    | table_header_group | table_footer_group | table_row
                    | table_cell | table_caption
                    => block,
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

    // CSS 2.1, Section 10 - Visual formatting model details

    ${predefined_type("width", "LengthOrPercentageOrAuto",
                      "computed::LPA_Auto",
                      "parse_non_negative")}
    ${predefined_type("height", "LengthOrPercentageOrAuto",
                      "computed::LPA_Auto",
                      "parse_non_negative")}

    ${predefined_type("min-width", "LengthOrPercentage",
                      "computed::LP_Length(Au(0))",
                      "parse_non_negative")}
    ${predefined_type("max-width", "LengthOrPercentageOrNone",
                      "computed::LPN_None",
                      "parse_non_negative")}

    ${new_style_struct("InheritedBox", is_inherited=True)}

    <%self:single_component_value name="line-height">
        #[deriving(Clone)]
        pub enum SpecifiedValue {
            SpecifiedNormal,
            SpecifiedLength(specified::Length),
            SpecifiedNumber(CSSFloat),
            // percentage are the same as em.
        }
        /// normal | <number> | <length> | <percentage>
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Option<SpecifiedValue> {
            match input {
                &ast::Number(ref value) if value.value >= 0.
                => Some(SpecifiedNumber(value.value)),
                &ast::Percentage(ref value) if value.value >= 0.
                => Some(SpecifiedLength(specified::Em(value.value / 100.))),
                &Dimension(ref value, ref unit) if value.value >= 0.
                => specified::Length::parse_dimension(value.value, unit.as_slice())
                    .map(SpecifiedLength),
                &Ident(ref value) if unsafe { value.to_ascii_nocheck().eq_ignore_case("normal".to_ascii_nocheck())}
                => Some(SpecifiedNormal),
                _ => None,
            }
        }
        pub mod computed_value {
            use super::super::{Au, CSSFloat};
            #[deriving(Eq, Clone)]
            pub enum T {
                Normal,
                Length(Au),
                Number(CSSFloat),
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { Normal }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                SpecifiedNormal => Normal,
                SpecifiedLength(value) => Length(computed::compute_Au(value, context)),
                SpecifiedNumber(value) => Number(value),
            }
        }
    </%self:single_component_value>

    ${switch_to_style_struct("Box")}

    <%self:single_component_value name="vertical-align">
        <% vertical_align_keywords = (
            "baseline sub super top text-top middle bottom text-bottom".split()) %>
        #[deriving(Clone)]
        pub enum SpecifiedValue {
            % for keyword in vertical_align_keywords:
                Specified_${to_rust_ident(keyword)},
            % endfor
            SpecifiedLengthOrPercentage(specified::LengthOrPercentage),
        }
        /// baseline | sub | super | top | text-top | middle | bottom | text-bottom
        /// | <percentage> | <length>
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Option<SpecifiedValue> {
            match input {
                &Ident(ref value) => {
                    // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
                    unsafe {
                        let tmp_for_lifetime = value.to_ascii_nocheck().to_lower();
                        let value_lower = tmp_for_lifetime.as_str_ascii();
                        match value_lower.as_slice() {
                            % for keyword in vertical_align_keywords:
                            "${keyword}" => Some(Specified_${to_rust_ident(keyword)}),
                            % endfor
                            _ => None,
                        }
                    }
                },
                _ => specified::LengthOrPercentage::parse_non_negative(input)
                     .map(SpecifiedLengthOrPercentage)
            }
        }
        pub mod computed_value {
            use super::super::{Au, CSSFloat};
            #[deriving(Eq, Clone)]
            pub enum T {
                % for keyword in vertical_align_keywords:
                    ${to_rust_ident(keyword)},
                % endfor
                Length(Au),
                Percentage(CSSFloat),
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { baseline }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                % for keyword in vertical_align_keywords:
                    Specified_${to_rust_ident(keyword)} => ${to_rust_ident(keyword)},
                % endfor
                SpecifiedLengthOrPercentage(value)
                => match computed::compute_LengthOrPercentage(value, context) {
                    computed::LP_Length(value) => Length(value),
                    computed::LP_Percentage(value) => Percentage(value)
                }
            }
        }
    </%self:single_component_value>


    // CSS 2.1, Section 11 - Visual effects
    ${single_keyword("overflow", "visible hidden")} // TODO: scroll auto

    ${switch_to_style_struct("InheritedBox")}

    // TODO: collapse. Well, do tables first.
    ${single_keyword("visibility", "visible hidden")}

    // CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

    ${switch_to_style_struct("Box")}

    <%self:longhand name="content">
            pub use to_computed_value = super::computed_as_specified;
            pub mod computed_value {
                #[deriving(Eq, Clone)]
                pub enum Content {
                    StringContent(~str),
                }
                #[deriving(Eq, Clone)]
                pub enum T {
                    normal,
                    none,
                    Content(~[Content]),
                }
            }
            pub type SpecifiedValue = computed_value::T;
            #[inline] pub fn get_initial_value() -> computed_value::T  { normal }

            // normal | none | [ <string> ]+
            // TODO: <uri>, <counter>, attr(<identifier>), open-quote, close-quote, no-open-quote, no-close-quote
            pub fn parse(input: &[ComponentValue], _base_url: &Url) -> Option<SpecifiedValue> {
                match one_component_value(input) {
                    Some(&Ident(ref keyword)) => {
                        unsafe {
                            let tmp_for_lifetime = keyword.to_ascii_nocheck().to_lower();
                            let keyword_lower = tmp_for_lifetime.as_str_ascii();
                            match keyword_lower.as_slice() {

                                "normal" => return Some(normal),
                                "none" => return Some(none),
                                _ => ()
                            }
                        }
                    },
                    _ => ()
                }
                let mut content = ~[];
                for component_value in input.skip_whitespace() {
                    match component_value {
                        &String(ref value)
                        => content.push(StringContent(value.to_owned())),
                        _ => return None  // invalid/unsupported value
                    }
                }
                Some(Content(content))
            }
    </%self:longhand>
    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    ${new_style_struct("Background", is_inherited=False)}
    ${predefined_type("background-color", "CSSColor",
                      "RGBA(RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */")}

    <%self:single_component_value name="background-image">
            // The computed value is the same as the specified value.
            pub use to_computed_value = super::computed_as_specified;
            pub mod computed_value {
                pub use extra::url::Url;
                pub type T = Option<Url>;
            }
            pub type SpecifiedValue = computed_value::T;
            #[inline] pub fn get_initial_value() -> SpecifiedValue {
                None
            }
            pub fn from_component_value(component_value: &ComponentValue, base_url: &Url) -> Option<SpecifiedValue> {
                match component_value {
                    &ast::URL(ref url) => {
                        let image_url = parse_url(url.as_slice(), Some(base_url.clone()));
                        Some(Some(image_url))
                    },
                    &ast::Ident(ref value) if unsafe {value.to_ascii_nocheck()}.eq_ignore_case(unsafe {"none".to_ascii_nocheck()})  => Some(None),
                    _ => None,
                }
            }
    </%self:single_component_value>


    ${new_style_struct("Color", is_inherited=True)}

    <%self:raw_longhand name="color">
        pub use to_computed_value = super::computed_as_specified;
        pub type SpecifiedValue = RGBA;
        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            RGBA { red: 0., green: 0., blue: 0., alpha: 1. }  /* black */
        }
        pub fn parse_specified(input: &[ComponentValue], _base_url: &Url)
                               -> Option<DeclaredValue<SpecifiedValue>> {
            match one_component_value(input).and_then(Color::parse) {
                Some(RGBA(rgba)) => Some(SpecifiedValue(rgba)),
                Some(CurrentColor) => Some(CSSWideKeyword(Inherit)),
                None => None,
            }
        }
    </%self:raw_longhand>

    // CSS 2.1, Section 15 - Fonts

    ${new_style_struct("Font", is_inherited=True)}

    <%self:longhand name="font-family">
        pub use to_computed_value = super::computed_as_specified;
        pub mod computed_value {
            #[deriving(Eq, Clone)]
            pub enum FontFamily {
                FamilyName(~str),
                // Generic
//                Serif,
//                SansSerif,
//                Cursive,
//                Fantasy,
//                Monospace,
            }
            pub type T = ~[FontFamily];
        }
        pub type SpecifiedValue = computed_value::T;
        #[inline] pub fn get_initial_value() -> computed_value::T { ~[FamilyName(~"serif")] }
        /// <familiy-name>#
        /// <familiy-name> = <string> | [ <ident>+ ]
        /// TODO: <generic-familiy>
        pub fn parse(input: &[ComponentValue], _base_url: &Url) -> Option<SpecifiedValue> {
            from_iter(input.skip_whitespace())
        }
        pub fn from_iter<'a>(mut iter: SkipWhitespaceIterator<'a>) -> Option<SpecifiedValue> {
            let mut result = ~[];
            macro_rules! add(
                ($value: expr, $b: expr) => {
                    {
                        result.push($value);
                        match iter.next() {
                            Some(&Comma) => (),
                            None => $b,
                            _ => return None,
                        }
                    }
                }
            )
            'outer: loop {
                match iter.next() {
                    // TODO: avoid copying strings?
                    Some(&String(ref value)) => add!(FamilyName(value.to_owned()), break 'outer),
                    Some(&Ident(ref value)) => {
                        // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
                        let value = value.as_slice();
                        unsafe {
                        let tmp_for_lifetime = value.to_ascii_nocheck().to_lower();
                        let value_lower = tmp_for_lifetime.as_str_ascii();
                        match value_lower.as_slice() {
//                            "serif" => add!(Serif, break 'outer),
//                            "sans-serif" => add!(SansSerif, break 'outer),
//                            "cursive" => add!(Cursive, break 'outer),
//                            "fantasy" => add!(Fantasy, break 'outer),
//                            "monospace" => add!(Monospace, break 'outer),
                            _ => {
                                let mut idents = ~[value];
                                loop {
                                    match iter.next() {
                                        Some(&Ident(ref value)) => idents.push(value.as_slice()),
                                        Some(&Comma) => {
                                            result.push(FamilyName(idents.connect(" ")));
                                            break
                                        },
                                        None => {
                                            result.push(FamilyName(idents.connect(" ")));
                                            break 'outer
                                        },
                                        _ => return None,
                                    }
                                }
                            }
                        }}
                    }
                    _ => return None,
                }
            }
            Some(result)
        }
    </%self:longhand>


    ${single_keyword("font-style", "normal italic oblique")}
    ${single_keyword("font-variant", "normal")}  // Add small-caps when supported

    <%self:single_component_value name="font-weight">
        #[deriving(Clone)]
        pub enum SpecifiedValue {
            Bolder,
            Lighter,
            % for weight in range(100, 901, 100):
                SpecifiedWeight${weight},
            % endfor
        }
        /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Option<SpecifiedValue> {
            match input {
                &Ident(ref value) => {
                    // FIXME: Workaround for https://github.com/mozilla/rust/issues/10683
                    unsafe {
                        let tmp_for_lifetime = value.to_ascii_nocheck().to_lower();
                        let value_lower = tmp_for_lifetime.as_str_ascii();
                        match value_lower.as_slice() {
                            "bold" => Some(SpecifiedWeight700),
                            "normal" => Some(SpecifiedWeight400),
                            "bolder" => Some(Bolder),
                            "lighter" => Some(Lighter),
                            _ => None,
                        }
                    }
                },
                &Number(ref value) => match value.int_value {
                    Some(100) => Some(SpecifiedWeight100),
                    Some(200) => Some(SpecifiedWeight200),
                    Some(300) => Some(SpecifiedWeight300),
                    Some(400) => Some(SpecifiedWeight400),
                    Some(500) => Some(SpecifiedWeight500),
                    Some(600) => Some(SpecifiedWeight600),
                    Some(700) => Some(SpecifiedWeight700),
                    Some(800) => Some(SpecifiedWeight800),
                    Some(900) => Some(SpecifiedWeight900),
                    _ => None,
                },
                _ => None
            }
        }
        pub mod computed_value {
            #[deriving(Eq, Clone)]
            pub enum T {
                % for weight in range(100, 901, 100):
                    Weight${weight},
                % endfor
            }
            impl T {
                pub fn is_bold(self) -> bool {
                    match self {
                        Weight900 | Weight800 | Weight700 | Weight600 => true,
                        _ => false
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { Weight400 }  // normal
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                % for weight in range(100, 901, 100):
                    SpecifiedWeight${weight} => Weight${weight},
                % endfor
                Bolder => match context.inherited_font_weight {
                    Weight100 => Weight400,
                    Weight200 => Weight400,
                    Weight300 => Weight400,
                    Weight400 => Weight700,
                    Weight500 => Weight700,
                    Weight600 => Weight900,
                    Weight700 => Weight900,
                    Weight800 => Weight900,
                    Weight900 => Weight900,
                },
                Lighter => match context.inherited_font_weight {
                    Weight100 => Weight100,
                    Weight200 => Weight100,
                    Weight300 => Weight100,
                    Weight400 => Weight100,
                    Weight500 => Weight100,
                    Weight600 => Weight400,
                    Weight700 => Weight400,
                    Weight800 => Weight700,
                    Weight900 => Weight700,
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
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(16)  // medium
        }
        #[inline]
        pub fn to_computed_value(_value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            // We already computed this element's font size; no need to compute it again.
            return context.font_size
        }
        /// <length> | <percentage>
        /// TODO: support <absolute-size> and <relative-size>
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Option<SpecifiedValue> {
            specified::LengthOrPercentage::parse_non_negative(input).map(|value| {
                match value {
                    specified::LP_Length(value) => value,
                    specified::LP_Percentage(value) => specified::Em(value),
                }
            })
        }
    </%self:single_component_value>

    // CSS 2.1, Section 16 - Text

    ${new_style_struct("InheritedText", is_inherited=True)}

    // TODO: initial value should be 'start' (CSS Text Level 3, direction-dependent.)
    ${single_keyword("text-align", "left right center justify")}

    ${new_style_struct("Text", is_inherited=False)}

    <%self:longhand name="text-decoration">
        pub use to_computed_value = super::computed_as_specified;
        #[deriving(Eq, Clone)]
        pub struct SpecifiedValue {
            underline: bool,
            overline: bool,
            line_through: bool,
            // 'blink' is accepted in the parser but ignored.
            // Just not blinking the text is a conforming implementation per CSS 2.1.
        }
        pub mod computed_value {
            pub type T = super::SpecifiedValue;
            pub static none: T = super::SpecifiedValue { underline: false, overline: false, line_through: false };
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            none
        }
        /// none | [ underline || overline || line-through || blink ]
        pub fn parse(input: &[ComponentValue], _base_url: &Url) -> Option<SpecifiedValue> {
            let mut result = SpecifiedValue {
                underline: false, overline: false, line_through: false,
            };
            let mut blink = false;
            let mut empty = true;
            for component_value in input.skip_whitespace() {
                match get_ident_lower(component_value) {
                    None => return None,
                    Some(keyword) => match keyword.as_slice() {
                        "underline" => if result.underline { return None }
                                      else { empty = false; result.underline = true },
                        "overline" => if result.overline { return None }
                                      else { empty = false; result.overline = true },
                        "line-through" => if result.line_through { return None }
                                          else { empty = false; result.line_through = true },
                        "blink" => if blink { return None }
                                   else { empty = false; blink = true },
                        "none" => return if empty { Some(result) } else { None },
                        _ => return None,
                    }
                }
            }
            if !empty { Some(result) } else { None }
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedText")}

    ${single_keyword("white-space", "normal pre")}

    // CSS 2.1, Section 17 - Tables
    ${new_style_struct("Table", is_inherited=False)}

    ${single_keyword("table-layout", "auto fixed")}

    // CSS 2.1, Section 18 - User interface
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
                    ${sub_property.ident}: Option<${sub_property.ident}::SpecifiedValue>,
                % endfor
            }
            pub fn parse(input: &[ComponentValue], base_url: &Url) -> Option<Longhands> {
                ${caller.body()}
            }
        }
    </%def>

    <%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
        <%self:shorthand name="${name}" sub_properties="${
                ' '.join(sub_property_pattern % side
                         for side in ['top', 'right', 'bottom', 'left'])}">
            let mut iter = input.skip_whitespace().map(|c| ${parser_function}(c, base_url));
            // zero or more than four values is invalid.
            // one value sets them all
            // two values set (top, bottom) and (left, right)
            // three values set top, (left, right) and bottom
            // four values set them in order
            let top = iter.next().unwrap_or(None);
            let right = iter.next().unwrap_or(top);
            let bottom = iter.next().unwrap_or(top);
            let left = iter.next().unwrap_or(right);
            if top.is_some() && right.is_some() && bottom.is_some() && left.is_some()
            && iter.next().is_none() {
                Some(Longhands {
                    % for side in ["top", "right", "bottom", "left"]:
                        ${to_rust_ident(sub_property_pattern % side)}: ${side},
                    % endfor
                })
            } else {
                None
            }
        </%self:shorthand>
    </%def>

    // TODO: other background-* properties
    <%self:shorthand name="background" sub_properties="background-color background-image">
                let mut color = None;
                let mut image = None;
                let mut any = false;

                for component_value in input.skip_whitespace() {
                    if color.is_none() {
                        match background_color::from_component_value(component_value, base_url) {
                            Some(v) => { color = Some(v); any = true; continue },
                            None => ()
                        }
                    }

                    if image.is_none() {
                        match background_image::from_component_value(component_value, base_url) {
                            Some(v) => { image = Some(v); any = true; continue },
                            None => (),
                        }
                    }
                    return None;
                }
                if any { Some(Longhands { background_color: color, background_image: image }) }
                else { None }
    </%self:shorthand>

    ${four_sides_shorthand("margin", "margin-%s", "margin_top::from_component_value")}
    ${four_sides_shorthand("padding", "padding-%s", "padding_top::from_component_value")}

    pub fn parse_color(value: &ComponentValue, _base_url: &Url) -> Option<specified::CSSColor> {
        specified::CSSColor::parse(value)
    }
    ${four_sides_shorthand("border-color", "border-%s-color", "parse_color")}
    ${four_sides_shorthand("border-style", "border-%s-style",
                           "border_top_style::from_component_value")}
    ${four_sides_shorthand("border-width", "border-%s-width", "parse_border_width")}

    pub fn parse_border(input: &[ComponentValue], base_url: &Url)
                     -> Option<(Option<specified::CSSColor>,
                                Option<border_top_style::SpecifiedValue>,
                                Option<specified::Length>)> {
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        for component_value in input.skip_whitespace() {
            if color.is_none() {
                match specified::CSSColor::parse(component_value) {
                    Some(c) => { color = Some(c); any = true; continue },
                    None => ()
                }
            }
            if style.is_none() {
                match border_top_style::from_component_value(component_value, base_url) {
                    Some(s) => { style = Some(s); any = true; continue },
                    None => ()
                }
            }
            if width.is_none() {
                match parse_border_width(component_value, base_url) {
                    Some(w) => { width = Some(w); any = true; continue },
                    None => ()
                }
            }
            return None
        }
        if any { Some((color, style, width)) } else { None }
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
                        ${"border_%s_%s: %s," % (side, prop, prop)}
                    % endfor
                % endfor
            }
        })
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
            if get_ident_lower(component_value).filtered(
                    |v| unsafe { v.to_ascii_nocheck() }.eq_ignore_case(unsafe {"normal".to_ascii_nocheck()})).is_some() {
                nb_normals += 1;
                continue;
            }
            if style.is_none() {
                match font_style::from_component_value(component_value, base_url) {
                    Some(s) => { style = Some(s); continue },
                    None => ()
                }
            }
            if weight.is_none() {
                match font_weight::from_component_value(component_value, base_url) {
                    Some(w) => { weight = Some(w); continue },
                    None => ()
                }
            }
            if variant.is_none() {
                match font_variant::from_component_value(component_value, base_url) {
                    Some(v) => { variant = Some(v); continue },
                    None => ()
                }
            }
            match font_size::from_component_value(component_value, base_url) {
                Some(s) => { size = Some(s); break },
                None => return None
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
            return None
        }
        let mut copied_iter = iter.clone();
        match copied_iter.next() {
            Some(&Delim('/')) => {
                iter = copied_iter;
                line_height = match iter.next() {
                    Some(v) => line_height::from_component_value(v, base_url),
                    _ => return None,
                };
                if line_height.is_none() { return None }
            }
            _ => ()
        }
        let family = font_family::from_iter(iter);
        if family.is_none() { return None }
        Some(Longhands {
            font_style: style,
            font_variant: variant,
            font_weight: weight,
            font_size: size,
            line_height: line_height,
            font_family: family
        })
    </%self:shorthand>

}


pub struct PropertyDeclarationBlock {
    important: Arc<~[PropertyDeclaration]>,
    normal: Arc<~[PropertyDeclaration]>,
}

impl<S: Encoder> Encodable<S> for PropertyDeclarationBlock {
    fn encode(&self, _: &mut S) {
    }
}


pub fn parse_style_attribute(input: &str, base_url: &Url) -> PropertyDeclarationBlock {
    parse_property_declaration_list(tokenize(input), base_url)
}


pub fn parse_property_declaration_list<I: Iterator<Node>>(input: I, base_url: &Url) -> PropertyDeclarationBlock {
    let mut important = ~[];
    let mut normal = ~[];
    for item in ErrorLoggerIterator(parse_declaration_list(input)) {
        match item {
            DeclAtRule(rule) => log_css_error(
                rule.location, format!("Unsupported at-rule in declaration list: @{:s}", rule.name)),
            Declaration(Declaration{ location: l, name: n, value: v, important: i}) => {
                // TODO: only keep the last valid declaration for a given name.
                let list = if i { &mut important } else { &mut normal };
                match PropertyDeclaration::parse(n, v, list, base_url) {
                    UnknownProperty => log_css_error(l, format!(
                        "Unsupported property: {}:{}", n, v.iter().to_css())),
                    InvalidValue => log_css_error(l, format!(
                        "Invalid value: {}:{}", n, v.iter().to_css())),
                    ValidDeclaration => (),
                }
            }
        }
    }
    PropertyDeclarationBlock {
        important: Arc::new(important),
        normal: Arc::new(normal),
    }
}


#[deriving(Clone)]
pub enum CSSWideKeyword {
    Initial,
    Inherit,
}

impl CSSWideKeyword {
    pub fn parse(input: &[ComponentValue]) -> Option<Option<CSSWideKeyword>> {
        one_component_value(input).and_then(get_ident_lower).and_then(|keyword| {
            match keyword.as_slice() {
                "initial" => Some(Some(Initial)),
                "inherit" => Some(Some(Inherit)),
                "unset" => Some(None),
                _ => None
            }
        })
    }
}


#[deriving(Clone)]
pub enum DeclaredValue<T> {
    SpecifiedValue(T),
    CSSWideKeyword(CSSWideKeyword),
}

#[deriving(Clone)]
pub enum PropertyDeclaration {
    % for property in LONGHANDS:
        ${property.ident}_declaration(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
}


enum PropertyDeclarationParseResult {
    UnknownProperty,
    InvalidValue,
    ValidDeclaration,
}


impl PropertyDeclaration {
    pub fn parse(name: &str, value: &[ComponentValue],
                 result_list: &mut ~[PropertyDeclaration],
                 base_url: &Url) -> PropertyDeclarationParseResult {
        // FIXME: local variable to work around Rust #10683
        let tmp_for_lifetime = unsafe {name.to_ascii_nocheck()}.to_lower();
        let name_lower = tmp_for_lifetime.as_str_ascii();
        match name_lower.as_slice() {
            % for property in LONGHANDS:
                "${property.name}" => result_list.push(${property.ident}_declaration(
                    match longhands::${property.ident}::parse_declared(value, base_url) {
                        Some(value) => value,
                        None => return InvalidValue,
                    }
                )),
            % endfor
            % for shorthand in SHORTHANDS:
                "${shorthand.name}" => match CSSWideKeyword::parse(value) {
                    Some(Some(keyword)) => {
                        % for sub_property in shorthand.sub_properties:
                            result_list.push(${sub_property.ident}_declaration(
                                CSSWideKeyword(keyword)
                            ));
                        % endfor
                    },
                    Some(None) => {
                        % for sub_property in shorthand.sub_properties:
                            result_list.push(${sub_property.ident}_declaration(
                                CSSWideKeyword(${
                                    "Inherit" if sub_property.style_struct.inherited else "Initial"})
                            ));
                        % endfor
                    },
                    None => match shorthands::${shorthand.ident}::parse(value, base_url) {
                        Some(result) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(${sub_property.ident}_declaration(
                                    match result.${sub_property.ident} {
                                        Some(value) => SpecifiedValue(value),
                                        None => CSSWideKeyword(Initial),
                                    }
                                ));
                            % endfor
                        },
                        None => return InvalidValue,
                    }
                },
            % endfor
            _ => return UnknownProperty,
        }
        ValidDeclaration
    }
}


pub mod style_structs {
    use super::longhands;
    % for style_struct in STYLE_STRUCTS:
        #[deriving(Eq, Clone)]
        pub struct ${style_struct.name} {
            % for longhand in style_struct.longhands:
                ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
        }
    % endfor
}

#[deriving(Eq, Clone)]
pub struct ComputedValues {
    % for style_struct in STYLE_STRUCTS:
        ${style_struct.name}: CowArc<style_structs::${style_struct.name}>,
    % endfor
    shareable: bool,
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
            RGBA(rgba) => rgba,
            CurrentColor => self.Color.get().color,
        }
    }
}

/// Returns the initial values for all style structs as defined by the specification.
pub fn initial_values() -> ComputedValues {
    ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.name}: CowArc::new(style_structs::${style_struct.name} {
                % for longhand in style_struct.longhands:
                    ${longhand.ident}: longhands::${longhand.ident}::get_initial_value(),
                % endfor
            }),
        % endfor
        shareable: true,
    }
}

/// Fast path for the function below. Only computes new inherited styles.
fn cascade_with_cached_declarations(applicable_declarations: &[MatchedProperty],
                                    shareable: bool,
                                    parent_style: &ComputedValues,
                                    cached_style: &ComputedValues,
                                    context: &computed::Context)
                                    -> ComputedValues {
    % for style_struct in STYLE_STRUCTS:
        % if style_struct.inherited:
            let mut style_${style_struct.name} = parent_style.${style_struct.name}.clone();
        % else:
            let style_${style_struct.name} = cached_style.${style_struct.name}.clone();
        % endif
    % endfor

    for sub_list in applicable_declarations.iter() {
        for declaration in sub_list.declarations.get().iter() {
            match *declaration {
                % for style_struct in STYLE_STRUCTS:
                    % if style_struct.inherited:
                        % for property in style_struct.longhands:
                            ${property.ident}_declaration(ref declared_value) => {
                                style_${style_struct.name}.get_mut().${property.ident} =
                                match *declared_value {
                                    SpecifiedValue(ref specified_value)
                                    => longhands::${property.ident}::to_computed_value(
                                        (*specified_value).clone(),
                                        context
                                    ),
                                    CSSWideKeyword(Initial)
                                    => longhands::${property.ident}::get_initial_value(),
                                    CSSWideKeyword(Inherit) => {
                                        // This is a bit slow, but this is rare so it shouldn't matter.
                                        // FIXME: is it still?
                                        parent_style.${style_struct.name}
                                                    .get()
                                                    .${property.ident}
                                                    .clone()
                                    }
                                }
                            }
                        % endfor
                    % endif
                % endfor
                _ => {}
            }
        }
    }

    ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.name}: style_${style_struct.name},
        % endfor
        shareable: shareable,
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
///   * `initial_values`: The initial set of CSS values as defined by the specification.
///
///   * `cached_style`: If present, cascading is short-circuited for everything but inherited
///     values and these values are used instead. Obviously, you must be careful when supplying
///     this that it is safe to only provide inherited declarations. If `parent_style` is `None`,
///     this is ignored.
///
/// Returns the computed values and a boolean indicating whether the result is cacheable.
pub fn cascade(applicable_declarations: &[MatchedProperty],
               shareable: bool,
               parent_style: Option< &ComputedValues >,
               initial_values: &ComputedValues,
               cached_style: Option< &ComputedValues >)
               -> (ComputedValues, bool) {
    let (is_root_element, inherited_style) = match parent_style {
        Some(parent_style) => (false, parent_style),
        None => (true, initial_values),
    };

    let mut context = {
        let inherited_font_style = inherited_style.Font.get();
        computed::Context {
            is_root_element: is_root_element,
            inherited_font_weight: inherited_font_style.font_weight,
            inherited_font_size: inherited_font_style.font_size,
            // To be overridden by applicable declarations:
            font_size: inherited_font_style.font_size,
            color: inherited_style.Color.get().color,
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
        ($style_struct: ident, $property: ident, $declared_value: expr) => {
            match *$declared_value {
                SpecifiedValue(specified_value) => specified_value,
                CSSWideKeyword(Initial) => longhands::$property::get_initial_value(),
                CSSWideKeyword(Inherit) => inherited_style.$style_struct.get().$property.clone(),
            }
        };
    )

    // Initialize `context`
    for sub_list in applicable_declarations.iter() {
        for declaration in sub_list.declarations.get().iter() {
            match *declaration {
                font_size_declaration(ref value) => {
                    context.font_size = match *value {
                        SpecifiedValue(specified_value) => computed::compute_Au_with_font_size(
                            specified_value, context.inherited_font_size),
                        CSSWideKeyword(Initial) => longhands::font_size::get_initial_value(),
                        CSSWideKeyword(Inherit) => context.inherited_font_size,
                    }
                }
                color_declaration(ref value) => {
                    context.color = get_specified!(Color, color, value);
                }
                position_declaration(ref value) => {
                    context.positioned = match get_specified!(Box, position, value) {
                        longhands::position::absolute | longhands::position::fixed => true,
                        _ => false,
                    }
                }
                float_declaration(ref value) => {
                    context.floated = get_specified!(Box, float, value) != longhands::float::none;
                }
                % for side in ["top", "right", "bottom", "left"]:
                    border_${side}_style_declaration(ref value) => {
                        context.border_${side}_present =
                        match get_specified!(Border, border_${side}_style, value) {
                            longhands::border_top_style::none |
                            longhands::border_top_style::hidden => false,
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
        let mut style_${style_struct.name} =
            % if style_struct.inherited:
                inherited_style
            % else:
                initial_values
            % endif
            .${style_struct.name}.clone();
    % endfor
    let mut cacheable = true;
    for sub_list in applicable_declarations.iter() {
        for declaration in sub_list.declarations.get().iter() {
            match *declaration {
                % for style_struct in STYLE_STRUCTS:
                    % for property in style_struct.longhands:
                        ${property.ident}_declaration(ref declared_value) => {
                            style_${style_struct.name}.get_mut().${property.ident} =
                            match *declared_value {
                                SpecifiedValue(ref specified_value)
                                => longhands::${property.ident}::to_computed_value(
                                    (*specified_value).clone(),
                                    &context
                                ),
                                CSSWideKeyword(Initial)
                                => longhands::${property.ident}::get_initial_value(),
                                CSSWideKeyword(Inherit) => {
                                    // This is a bit slow, but this is rare so it shouldn't matter.
                                    // FIXME: is it still?
                                    cacheable = false;
                                    inherited_style.${style_struct.name}
                                                   .get()
                                                   .${property.ident}
                                                   .clone()
                                }
                            }
                        }
                    % endfor
                % endfor
            }
        }
    }

    (ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.name}: style_${style_struct.name},
        % endfor
        shareable: shareable,
    }, cacheable)
}


// Only re-export the types for computed values.
pub mod computed_values {
    % for property in LONGHANDS:
        pub use ${property.ident} = super::longhands::${property.ident}::computed_value;
    % endfor
    // Don't use a side-specific name needlessly:
    pub use border_style = super::longhands::border_top_style::computed_value;

    pub use cssparser::RGBA;
    pub use super::common_types::computed::{
        LengthOrPercentage, LP_Length, LP_Percentage,
        LengthOrPercentageOrAuto, LPA_Length, LPA_Percentage, LPA_Auto,
        LengthOrPercentageOrNone, LPN_Length, LPN_Percentage, LPN_None};
}
