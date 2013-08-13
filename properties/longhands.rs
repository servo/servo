/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */


pub use std::ascii::{to_ascii_lower, eq_ignore_ascii_case};
pub use std::option;
pub use cssparser::*;
pub use CSSColor = cssparser::Color;
pub use parsing_utils::*;
pub use super::common_types::specified;
pub use super::common_types;


macro_rules! single_keyword(
    ($property_name: ident, $( $lower_case_keyword_string: pat => $variant: ident ),+ ) => {
        mod $property_name {
            use super::*;
            enum SpecifiedValue {
                $( $variant ),+
            }
            fn parse(input: &[ComponentValue]) -> option::Option<SpecifiedValue> {
                do one_component_value(input).chain(get_ident_lower).chain |keyword| {
                    match keyword.as_slice() {
                        $( $lower_case_keyword_string => option::Some($variant) ),+ ,
                        _ => option::None,
                    }
                }
            }
        }
    };
)


macro_rules! single_type(
    ($property_name: ident, $type_: ident) => {
        single_type!($property_name, $type_, $type_::parse)
    };
    ($property_name: ident, $type_: ty, $parse_function: expr) => {
        mod $property_name {
            use super::*;
            type SpecifiedValue = $type_;
            fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
                one_component_value(input).chain($parse_function)
            }
        }
    };
)



// CSS 2.1, Section 8 - Box model

single_type!(margin_top, specified::LengthOrPercentageOrAuto,
                         specified::LengthOrPercentageOrAuto::parse)
single_type!(margin_right, specified::LengthOrPercentageOrAuto,
                           specified::LengthOrPercentageOrAuto::parse)
single_type!(margin_bottom, specified::LengthOrPercentageOrAuto,
                            specified::LengthOrPercentageOrAuto::parse)
single_type!(margin_left, specified::LengthOrPercentageOrAuto,
                          specified::LengthOrPercentageOrAuto::parse)

single_type!(padding_top, specified::LengthOrPercentage,
                          specified::LengthOrPercentage::parse_non_negative)
single_type!(padding_right, specified::LengthOrPercentage,
                            specified::LengthOrPercentage::parse_non_negative)
single_type!(padding_bottom, specified::LengthOrPercentage,
                             specified::LengthOrPercentage::parse_non_negative)
single_type!(padding_left, specified::LengthOrPercentage,
                           specified::LengthOrPercentage::parse_non_negative)

single_type!(border_top_color, CSSColor)
single_type!(border_right_color, CSSColor)
single_type!(border_bottom_color, CSSColor)
single_type!(border_left_color, CSSColor)

pub fn parse_border_width(component_value: &ComponentValue) -> Option<specified::Length> {
    match component_value {
        &Ident(ref value) => match to_ascii_lower(value.as_slice()).as_slice() {
            "thin" => Some(specified::Length::from_px(1.)),
            "medium" => Some(specified::Length::from_px(3.)),
            "thick" => Some(specified::Length::from_px(5.)),
            _ => None
        },
        _ => specified::Length::parse_non_negative(component_value)
    }
}

single_type!(border_top_width, specified::Length, parse_border_width)
single_type!(border_right_width, specified::Length, parse_border_width)
single_type!(border_bottom_width, specified::Length, parse_border_width)
single_type!(border_left_width, specified::Length, parse_border_width)

// CSS 2.1, Section 9 - Visual formatting model

// TODO: don’t parse values we don’t support
single_keyword!(display,
    "inline" => Inline,
    "block" => Block,
    "list-item" => ListItem,
    "inline-block" => InlineBlock,
    "table" => Table,
    "inline-table" => InlineTable,
    "table-row-group" => TableRowGroup,
    "table-header-group" => TableHeaderGroup,
    "table-footer-group" => TableFooterGroup,
    "table-row" => TableRow,
    "table-column-group" => TableColumnGroup,
    "table-column" => TableColumn,
    "table-cell" => TableCell,
    "table-caption" => TableCaption,
    "none" => None
)

single_keyword!(position,
    "static" => Static, "absolute" => Absolute, "relative" => Relative, "fixed" => Fixed)
single_keyword!(float, "left" => Left, "right" => Right, "none" => None)
single_keyword!(clear, "left" => Left, "right" => Right, "none" => None, "both" => Both)


// CSS 2.1, Section 10 - Visual formatting model details

single_type!(width, specified::LengthOrPercentageOrAuto,
                    specified::LengthOrPercentageOrAuto::parse_non_negative)
single_type!(height, specified::LengthOrPercentageOrAuto,
                     specified::LengthOrPercentageOrAuto::parse_non_negative)

mod line_height {
    use super::*;
    enum SpecifiedValue {
        Normal,
        Length(specified::Length),
        Percentage(common_types::Float),
        Number(common_types::Float),
    }
    /// normal | <number> | <length> | <percentage>
    fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
        match one_component_value(input) {
            Some(&ast::Number(ref value)) if value.value >= 0.
            => Some(Number(value.value)),
            Some(&ast::Percentage(ref value)) if value.value >= 0.
            => Some(Percentage(value.value)),
            Some(&Dimension(ref value, ref unit)) if value.value >= 0.
            => specified::Length::parse_dimension(value.value, unit.as_slice()).map_move(Length),
            Some(&Ident(ref value)) if eq_ignore_ascii_case(value.as_slice(), "auto")
            => Some(Normal),
            _ => None,
        }
    }
}


// CSS 2.1, Section 11 - Visual effects

// CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

// CSS 2.1, Section 13 - Paged media

// CSS 2.1, Section 14 - Colors and Backgrounds

single_type!(background_color, CSSColor)
single_type!(color, CSSColor)

// CSS 2.1, Section 15 - Fonts

mod font_family {
    use super::*;
    enum FontFamily {
        FamilyName(~str),
        // Generic
        Serif,
        SansSerif,
        Cursive,
        Fantasy,
        Monospace,
    }
    type SpecifiedValue = ~[FontFamily];
    /// <familiy-name>#
    /// <familiy-name> = <string> | [ <ident>+ ]
    /// TODO: <generic-familiy>
    fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
        let mut result = ~[];
        let mut iter = input.skip_whitespace();
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
                    match to_ascii_lower(value).as_slice() {
                        "serif" => add!(Serif),
                        "sans-serif" => add!(SansSerif),
                        "cursive" => add!(Cursive),
                        "fantasy" => add!(Fantasy),
                        "monospace" => add!(Monospace),
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
}

single_keyword!(font_style, "normal" => Normal, "italic" => Italic, "oblique" => Oblique)
single_keyword!(font_variant, "normal" => Normal, "small-caps" => SmallCaps)

mod font_weight {
    use super::*;
    enum SpecifiedValue {
        Bolder,
        Lighther,
        Weight100,
        Weight200,
        Weight300,
        Weight400,
        Weight500,
        Weight600,
        Weight700,
        Weight800,
        Weight900,
    }
    /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
    fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
        match one_component_value(input) {
            Some(&Ident(ref value)) => match to_ascii_lower(value.as_slice()).as_slice() {
                "bold" => Some(Weight700),
                "normal" => Some(Weight400),
                "bolder" => Some(Bolder),
                "lighter" => Some(Lighther),
                _ => None,
            },
            Some(&Number(ref value)) => match value.int_value {
                Some(100) => Some(Weight100),
                Some(200) => Some(Weight200),
                Some(300) => Some(Weight300),
                Some(400) => Some(Weight400),
                Some(500) => Some(Weight500),
                Some(600) => Some(Weight600),
                Some(700) => Some(Weight700),
                Some(800) => Some(Weight800),
                Some(900) => Some(Weight900),
                _ => None,
            },
            _ => None
        }
    }
}

mod font_size {
    use super::*;
    type SpecifiedValue = specified::Length;  // Percentages are the same as em.
    /// <length> | <percentage>
    /// TODO: support <absolute-size> and <relative-size>
    fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
        do one_component_value(input).chain(specified::LengthOrPercentage::parse_non_negative)
        .map_move |value| {
            match value {
                specified::Length(value) => value,
                specified::Percentage(value) => specified::Em(value),
            }
        }
    }
}

// CSS 2.1, Section 16 - Text

single_keyword!(text_align, "left" => Left, "right" => Right,
                            "center" => Center, "justify" => Justify)

mod text_decoration {
    use super::*;
    struct SpecifiedValue {
        underline: bool,
        overline: bool,
        line_through: bool,
        // 'blink' is accepted in the parser but ignored.
        // Just not blinking the text is a conforming implementation per CSS 2.1.
    }
    /// none | [ underline || overline || line-through || blink ]
    fn parse(input: &[ComponentValue]) -> Option<SpecifiedValue> {
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
}

// CSS 2.1, Section 17 - Tables

// CSS 2.1, Section 18 - User interface
