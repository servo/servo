/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Position", inherited=False) %>

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type(side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto")}
% endfor

<%helpers:longhand name="z-index">
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
                match *self {
                    T::Auto => dest.write_str("auto"),
                    T::Number(number) => write!(dest, "{}", number),
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
            specified::parse_integer(input).map(computed_value::T::Number)
        }
    }
</%helpers:longhand>

// CSS Flexible Box Layout Module Level 1
// http://www.w3.org/TR/css3-flexbox/

// Flex container properties
${helpers.single_keyword("flex-direction", "row row-reverse column column-reverse", experimental=True)}

// Flex item properties
${helpers.predefined_type("flex-grow", "Number", "0.0", "parse_non_negative", products="gecko")}

${helpers.predefined_type("flex-shrink", "Number", "1.0", "parse_non_negative", products="gecko")}

// https://drafts.csswg.org/css-flexbox/#propdef-order
<%helpers:longhand name="order">
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}

    pub type SpecifiedValue = computed_value::T;

    pub mod computed_value {
        pub type T = i32;
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        0
    }

    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        specified::parse_integer(input)
    }
</%helpers:longhand>

${helpers.predefined_type("flex-basis",
                          "LengthOrPercentageOrAutoOrContent",
                          "computed::LengthOrPercentageOrAutoOrContent::Auto")}

${helpers.single_keyword("flex-wrap", "nowrap wrap wrap-reverse", products="gecko")}

${helpers.predefined_type("min-width",
                          "LengthOrPercentage",
                          "computed::LengthOrPercentage::Length(Au(0))",
                          "parse_non_negative")}
${helpers.predefined_type("max-width",
                          "LengthOrPercentageOrNone",
                          "computed::LengthOrPercentageOrNone::None",
                          "parse_non_negative")}

${helpers.predefined_type("min-height",
                          "LengthOrPercentage",
                          "computed::LengthOrPercentage::Length(Au(0))",
                          "parse_non_negative")}
${helpers.predefined_type("max-height",
                          "LengthOrPercentageOrNone",
                          "computed::LengthOrPercentageOrNone::None",
                          "parse_non_negative")}

${helpers.single_keyword("box-sizing",
                         "content-box border-box")}

// CSS Image Values and Replaced Content Module Level 3
// https://drafts.csswg.org/css-images-3/
${helpers.single_keyword("object-fit", "fill contain cover none scale-down", products="gecko")}
