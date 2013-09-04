/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

use std::ascii::StrAsciiExt;
pub use std::iterator;
pub use cssparser::*;
pub use style::errors::{ErrorLoggerIterator, log_css_error};
pub use style::parsing_utils::*;
pub use self::common_types::*;

pub mod common_types;


<%!

def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static"]:  # Rust keywords
        name += "_"
    return name

class Longhand(object):
    def __init__(self, name):
        self.name = name
        self.ident = to_rust_ident(name)


class Shorthand(object):
    def __init__(self, name, sub_properties):
        self.name = name
        self.ident = to_rust_ident(name)
        self.sub_properties = [Longhand(s) for s in sub_properties]

LONGHANDS = []
SHORTHANDS = []
INHERITED = set()

%>

pub mod longhands {
    pub use super::*;
    pub use std;

    <%def name="longhand(name, inherited=False, no_super=False)">
    <%
        property = Longhand(name)
        LONGHANDS.append(property)
        if inherited:
            INHERITED.add(name)
    %>
        pub mod ${property.ident} {
            % if not no_super:
                use super::*;
            % endif
            ${caller.body()}
        }
    </%def>

    <%def name="single_component_value(name, inherited=False)">
        <%self:longhand name="${name}" inherited="${inherited}">
            ${caller.body()}
            pub fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
                one_component_value(input).chain(from_component_value)
            }
        </%self:longhand>
    </%def>

    <%def name="single_keyword(name, values, inherited=False)">
        <%self:single_component_value name="${name}" inherited="${inherited}">
            // The computed value is the same as the specified value.
            pub use to_computed_value = std::util::id;
            pub enum SpecifiedValue {
                % for value in values.split():
                    ${to_rust_ident(value)},
                % endfor
            }
            pub type ComputedValue = SpecifiedValue;
            #[inline] pub fn get_initial_value() -> ComputedValue {
                ${to_rust_ident(values.split()[0])}
            }
            pub fn from_component_value(v: &ComponentValue) -> Option<SpecifiedValue> {
                do get_ident_lower(v).chain |keyword| {
                    match keyword.as_slice() {
                        % for value in values.split():
                            "${value}" => Some(${to_rust_ident(value)}),
                        % endfor
                        _ => None,
                    }
                }
            }
        </%self:single_component_value>
    </%def>

    <%def name="predefined_type(name, type, initial_value, parse_method='parse', inherited=False)">
        <%self:longhand name="${name}" inherited="${inherited}">
            pub use to_computed_value = super::super::common_types::computed::compute_${type};
            pub type SpecifiedValue = specified::${type};
            pub type ComputedValue = computed::${type};
            #[inline] pub fn get_initial_value() -> ComputedValue { ${initial_value} }
            pub fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
                one_component_value(input).chain(specified::${type}::${parse_method})
            }
        </%self:longhand>
    </%def>


    // CSS 2.1, Section 8 - Box model

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("margin-" + side, "LengthOrPercentageOrAuto",
                          "computed::LPA_Length(computed::Length(0))")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("padding-" + side, "LengthOrPercentage",
                          "computed::LP_Length(computed::Length(0))",
                          "parse_non_negative")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("border-%s-color" % side, "CSSColor", "CurrentColor")}
    % endfor

    // dotted dashed double groove ridge insed outset hidden
    ${single_keyword("border-top-style", "none solid")}
    % for side in ["right", "bottom", "left"]:
        <%self:longhand name="border-${side}-style", no_super="True">
            pub use super::border_top_style::*;
            pub type SpecifiedValue = super::border_top_style::SpecifiedValue;
            pub type ComputedValue = super::border_top_style::ComputedValue;
        </%self:longhand>
    % endfor

    pub fn parse_border_width(component_value: &ComponentValue) -> Option<specified::Length> {
        match component_value {
            &Ident(ref value) => match value.to_ascii_lower().as_slice() {
                "thin" => Some(specified::Length::from_px(1.)),
                "medium" => Some(specified::Length::from_px(3.)),
                "thick" => Some(specified::Length::from_px(5.)),
                _ => None
            },
            _ => specified::Length::parse_non_negative(component_value)
        }
    }
    % for side in ["top", "right", "bottom", "left"]:
        <%self:longhand name="border-${side}-width">
            pub type SpecifiedValue = specified::Length;
            pub type ComputedValue = computed::Length;
            #[inline] pub fn get_initial_value() -> ComputedValue {
                computed::Length(3 * 60)  // medium
            }
            pub fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
                one_component_value(input).chain(parse_border_width)
            }
            pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                  -> ComputedValue {
                if context.has_border_${side} { computed::compute_Length(value, context) }
                else { computed::Length(0) }
            }
        </%self:longhand>
    % endfor

    // CSS 2.1, Section 9 - Visual formatting model

    // TODO: don't parse values we don't support
    ${single_keyword("display",
        "inline block list-item inline-block none "
    )}
//        "table inline-table table-row-group table-header-group table-footer-group "
//        "table-row table-column-group table-column table-cell table-caption"

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

    <%self:single_component_value name="line-height">
        pub enum SpecifiedValue {
            SpecifiedNormal,
            SpecifiedLength(specified::Length),
            SpecifiedNumber(Float),
            // percentage are the same as em.
        }
        /// normal | <number> | <length> | <percentage>
        pub fn from_component_value(input: &ComponentValue) -> Option<SpecifiedValue> {
            match input {
                &ast::Number(ref value) if value.value >= 0.
                => Some(SpecifiedNumber(value.value)),
                &ast::Percentage(ref value) if value.value >= 0.
                => Some(SpecifiedLength(specified::Em(value.value))),
                &Dimension(ref value, ref unit) if value.value >= 0.
                => specified::Length::parse_dimension(value.value, unit.as_slice())
                    .map_move(SpecifiedLength),
                &Ident(ref value) if value.eq_ignore_ascii_case("normal")
                => Some(SpecifiedNormal),
                _ => None,
            }
        }
        pub enum ComputedValue {
            Normal,
            Length(computed::Length),
            Number(Float),
        }
        #[inline] pub fn get_initial_value() -> ComputedValue { Normal }
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> ComputedValue {
            match value {
                SpecifiedNormal => Normal,
                SpecifiedLength(value) => Length(computed::compute_Length(value, context)),
                SpecifiedNumber(value) => Number(value),
            }
        }
    </%self:single_component_value>

    // CSS 2.1, Section 11 - Visual effects

    // CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    ${predefined_type("background-color", "CSSColor",
                      "RGBA(RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */")}
    ${predefined_type("color", "CSSColor",
                      "RGBA(RGBA { red: 0., green: 0., blue: 0., alpha: 1. }) /* black */",
                      inherited=True)}

    // CSS 2.1, Section 15 - Fonts

    <%self:longhand name="font-family" inherited="True">
        pub use to_computed_value = std::util::id;
        enum FontFamily {
            FamilyName(~str),
            // Generic
//            Serif,
//            SansSerif,
//            Cursive,
//            Fantasy,
//            Monospace,
        }
        pub type SpecifiedValue = ~[FontFamily];
        pub type ComputedValue = SpecifiedValue;
        #[inline] pub fn get_initial_value() -> ComputedValue { ~[FamilyName(~"serif")] }
        /// <familiy-name>#
        /// <familiy-name> = <string> | [ <ident>+ ]
        /// TODO: <generic-familiy>
        pub fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
            from_iter(input.skip_whitespace())
        }
        pub fn from_iter<'a>(mut iter: SkipWhitespaceIterator<'a>) -> Option<SpecifiedValue> {
            let mut result = ~[];
            macro_rules! add(
                ($value: expr) => {
                    {
                        result.push($value);
                        match iter.next() {
                            Some(&Comma) => (),
                            None => break 'outer,
                            _ => return None,
                        }
                    }
                }
            )
            'outer: loop {
                match iter.next() {
                    // TODO: avoid copying strings?
                    Some(&String(ref value)) => add!(FamilyName(value.to_owned())),
                    Some(&Ident(ref value)) => {
                        let value = value.as_slice();
                        match value.to_ascii_lower().as_slice() {
//                            "serif" => add!(Serif),
//                            "sans-serif" => add!(SansSerif),
//                            "cursive" => add!(Cursive),
//                            "fantasy" => add!(Fantasy),
//                            "monospace" => add!(Monospace),
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
                        }
                    }
                    _ => return None,
                }
            }
            Some(result)
        }
    </%self:longhand>


    ${single_keyword("font-style", "normal italic oblique", inherited=True)}
    ${single_keyword("font-variant", "normal", inherited=True)}  // Add small-caps when supported

    <%self:single_component_value name="font-weight" inherited="True">
        pub enum SpecifiedValue {
            Bolder,
            Lighther,
            % for weight in range(100, 901, 100):
                SpecifiedWeight${weight},
            % endfor
        }
        /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
        pub fn from_component_value(input: &ComponentValue) -> Option<SpecifiedValue> {
            match input {
                &Ident(ref value) => match value.to_ascii_lower().as_slice() {
                    "bold" => Some(SpecifiedWeight700),
                    "normal" => Some(SpecifiedWeight400),
                    "bolder" => Some(Bolder),
                    "lighter" => Some(Lighther),
                    _ => None,
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
        pub enum ComputedValue {
            % for weight in range(100, 901, 100):
                Weight${weight},
            % endfor
        }
        #[inline] pub fn get_initial_value() -> ComputedValue { Weight400 }  // normal
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> ComputedValue {
            match value {
                % for weight in range(100, 901, 100):
                    SpecifiedWeight${weight} => Weight${weight},
                % endfor
                Bolder => match context.font_weight {
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
                Lighther => match context.font_weight {
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

    <%self:single_component_value name="font-size" inherited="True">
        pub use to_computed_value = super::super::common_types::computed::compute_Length;
        pub type SpecifiedValue = specified::Length;  // Percentages are the same as em.
        pub type ComputedValue = computed::Length;
        #[inline] pub fn get_initial_value() -> ComputedValue {
            computed::Length(16 * 60)  // medium
        }
        /// <length> | <percentage>
        /// TODO: support <absolute-size> and <relative-size>
        pub fn from_component_value(input: &ComponentValue) -> Option<SpecifiedValue> {
            do specified::LengthOrPercentage::parse_non_negative(input).map_move |value| {
                match value {
                    specified::LP_Length(value) => value,
                    specified::LP_Percentage(value) => specified::Em(value),
                }
            }
        }
    </%self:single_component_value>

    // CSS 2.1, Section 16 - Text

    // TODO: initial value should be 'start' (CSS Text Level 3, direction-dependent.)
    ${single_keyword("text-align", "left right center justify", inherited=True)}

    <%self:longhand name="text-decoration">
        pub use to_computed_value = std::util::id;
        pub struct SpecifiedValue {
            underline: bool,
            overline: bool,
            line_through: bool,
            // 'blink' is accepted in the parser but ignored.
            // Just not blinking the text is a conforming implementation per CSS 2.1.
        }
        pub type ComputedValue = SpecifiedValue;
        #[inline] pub fn get_initial_value() -> ComputedValue {
            SpecifiedValue { underline: false, overline: false, line_through: false }  // none
        }
        /// none | [ underline || overline || line-through || blink ]
        pub fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
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

    // CSS 2.1, Section 17 - Tables

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
            struct Longhands {
                % for sub_property in shorthand.sub_properties:
                    ${sub_property.ident}: Option<${sub_property.ident}::SpecifiedValue>,
                % endfor
            }
            pub fn parse(input: &[ComponentValue]) -> Option<Longhands> {
                ${caller.body()}
            }
        }
    </%def>

    <%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
        <%self:shorthand name="${name}" sub_properties="${
                ' '.join(sub_property_pattern % side
                         for side in ['top', 'right', 'bottom', 'left'])}">
            let mut iter = input.skip_whitespace().map(${parser_function});
            // zero or more than four values is invalid.
            // one value sets them all
            // two values set (top, bottom) and (left, right)
            // three values set top, (left, right) and bottom
            // four values set them in order
            let top = iter.next().unwrap_or_default(None);
            let right = iter.next().unwrap_or_default(top);
            let bottom = iter.next().unwrap_or_default(top);
            let left = iter.next().unwrap_or_default(right);
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
    <%self:shorthand name="background" sub_properties="background-color">
        do one_component_value(input).chain(specified::CSSColor::parse).map_move |color| {
            Longhands { background_color: Some(color) }
        }
    </%self:shorthand>

    ${four_sides_shorthand("border-color", "border-%s-color", "specified::CSSColor::parse")}
    ${four_sides_shorthand("border-style", "border-%s-style",
                           "border_top_style::from_component_value")}
    ${four_sides_shorthand("border-width", "border-%s-width", "parse_border_width")}

    pub fn parse_border(input: &[ComponentValue])
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
                    Some(c) => { color = Some(c); any = true; loop },
                    None => ()
                }
            }
            if style.is_none() {
                match border_top_style::from_component_value(component_value) {
                    Some(s) => { style = Some(s); any = true; loop },
                    None => ()
                }
            }
            if width.is_none() {
                match parse_border_width(component_value) {
                    Some(w) => { width = Some(w); any = true; loop },
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
            do parse_border(input).map_move |(color, style, width)| {
                Longhands {
                    % for prop in ["color", "style", "width"]:
                        ${"border_%s_%s: %s," % (side, prop, prop)}
                    % endfor
                }
            }
        </%self:shorthand>
    % endfor

    <%self:shorthand name="border" sub_properties="${' '.join(
        'border-%s-%s' % (side, prop)
        for side in ['top', 'right', 'bottom', 'left']
        for prop in ['color', 'style', 'width']
    )}">
        do parse_border(input).map_move |(color, style, width)| {
            Longhands {
                % for side in ["top", "right", "bottom", "left"]:
                    % for prop in ["color", "style", "width"]:
                        ${"border_%s_%s: %s," % (side, prop, prop)}
                    % endfor
                % endfor
            }
        }
    </%self:shorthand>

}


pub struct PropertyDeclarationBlock {
    important: ~[PropertyDeclaration],
    normal: ~[PropertyDeclaration],
}


pub fn parse_property_declaration_list(input: ~[Node]) -> PropertyDeclarationBlock {
    let mut important = ~[];
    let mut normal = ~[];
    for item in ErrorLoggerIterator(parse_declaration_list(input.move_iter())) {
        match item {
            Decl_AtRule(rule) => log_css_error(
                rule.location, fmt!("Unsupported at-rule in declaration list: @%s", rule.name)),
            Declaration(Declaration{ location: l, name: n, value: v, important: i}) => {
                let list = if i { &mut important } else { &mut normal };
                if !PropertyDeclaration::parse(n, v, list) {
                    log_css_error(l, "Invalid property declaration")
                }
            }
        }
    }
    PropertyDeclarationBlock { important: important, normal: normal }
}


pub enum CSSWideKeyword {
    Initial,
    Inherit,
    Unset,
}

impl CSSWideKeyword {
    pub fn parse(input: &[ComponentValue]) -> Option<CSSWideKeyword> {
        do one_component_value(input).chain(get_ident_lower).chain |keyword| {
            match keyword.as_slice() {
                "initial" => Some(Initial),
                "inherit" => Some(Inherit),
                "unset" => Some(Unset),
                _ => None
            }
        }
    }
}

pub enum DeclaredValue<T> {
    SpecifiedValue(T),
    CSSWideKeyword(CSSWideKeyword),
}

pub enum PropertyDeclaration {
    % for property in LONGHANDS:
        ${property.ident}_declaration(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
}

impl PropertyDeclaration {
    pub fn parse(name: &str, value: &[ComponentValue],
                 result_list: &mut ~[PropertyDeclaration]) -> bool {
        match name.to_ascii_lower().as_slice() {
            % for property in LONGHANDS:
                "${property.name}" => result_list.push(${property.ident}_declaration(
                    match CSSWideKeyword::parse(value) {
                        Some(keyword) => CSSWideKeyword(keyword),
                        None => match longhands::${property.ident}::parse(value) {
                            Some(value) => SpecifiedValue(value),
                            None => return false,
                        }
                    }
                )),
            % endfor
            % for shorthand in SHORTHANDS:
                "${shorthand.name}" => match CSSWideKeyword::parse(value) {
                    Some(keyword) => {
                        % for sub_property in shorthand.sub_properties:
                            result_list.push(${sub_property.ident}_declaration(
                                CSSWideKeyword(keyword)
                            ));
                        % endfor
                    },
                    None => match shorthands::${shorthand.ident}::parse(value) {
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
                        None => return false,
                    }
                },
            % endfor
            _ => return false,  // Unknown property
        }
        true
    }
}
