/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import ALL_SIZES %>

<% data.new_style_struct("Position", inherited=False) %>

% for side in ["top", "right", "bottom", "left"]:
    ${helpers.predefined_type(side, "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto",
                              animatable=True)}
% endfor

<%helpers:longhand name="z-index" animatable="True">
    use values::NoViewportPercentage;
    use values::computed::ComputedValueAsSpecified;

    impl ComputedValueAsSpecified for SpecifiedValue {}
    impl NoViewportPercentage for SpecifiedValue {}
    pub type SpecifiedValue = computed_value::T;
    pub mod computed_value {
        use std::fmt;
        use style_traits::ToCss;

        #[derive(PartialEq, Clone, Eq, Copy, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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
${helpers.single_keyword("flex-direction", "row row-reverse column column-reverse",
                         animatable=False)}

${helpers.single_keyword("flex-wrap", "nowrap wrap wrap-reverse",
                         animatable=False)}

// FIXME(stshine): The type of 'justify-content' and 'align-content' is uint16_t in gecko
// FIXME(stshine): Its higher bytes are used to store fallback value. Disable them in geckolib for now
${helpers.single_keyword("justify-content", "flex-start flex-end center space-between space-around",
                         gecko_constant_prefix="NS_STYLE_JUSTIFY",
                         products="servo",
                         animatable=False)}

// FIXME(heycam): Disable align-items in geckolib since we don't support the Gecko initial value
// 'normal' yet.
${helpers.single_keyword("align-items", "stretch flex-start flex-end center baseline",
                         need_clone=True,
                         gecko_constant_prefix="NS_STYLE_ALIGN",
                         animatable=False,
                         products="servo")}

${helpers.single_keyword("align-content", "stretch flex-start flex-end center space-between space-around",
                         gecko_constant_prefix="NS_STYLE_ALIGN",
                         products="servo",
                         animatable=False)}

// Flex item properties
${helpers.predefined_type("flex-grow", "Number",
                          "0.0", "parse_non_negative",
                          animatable=True)}

${helpers.predefined_type("flex-shrink", "Number",
                          "1.0", "parse_non_negative",
                          animatable=True)}

${helpers.single_keyword("align-self", "auto stretch flex-start flex-end center baseline",
                         need_clone=True,
                         gecko_constant_prefix="NS_STYLE_ALIGN",
                         animatable=False)}

// https://drafts.csswg.org/css-flexbox/#propdef-order
<%helpers:longhand name="order" products="servo" animatable="True">
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

// FIXME: This property should be animatable.
${helpers.predefined_type("flex-basis",
                          "LengthOrPercentageOrAutoOrContent",
                          "computed::LengthOrPercentageOrAutoOrContent::Auto",
                          animatable=False)}

% for (size, logical) in ALL_SIZES:
    // width, height, block-size, inline-size
    ${helpers.predefined_type("%s" % size,
                              "LengthOrPercentageOrAuto",
                              "computed::LengthOrPercentageOrAuto::Auto",
                              "parse_non_negative",
                              animatable=True, logical = logical)}

    // min-width, min-height, min-block-size, min-inline-size
    ${helpers.predefined_type("min-%s" % size,
                              "LengthOrPercentage",
                              "computed::LengthOrPercentage::Length(Au(0))",
                              "parse_non_negative",
                              animatable=True, logical = logical)}

    // max-width, max-height, max-block-size, max-inline-size
    ${helpers.predefined_type("max-%s" % size,
                              "LengthOrPercentageOrNone",
                              "computed::LengthOrPercentageOrNone::None",
                              "parse_non_negative",
                              animatable=True, logical = logical)}
% endfor

${helpers.single_keyword("box-sizing",
                         "content-box border-box",
                         animatable=False)}

// CSS Image Values and Replaced Content Module Level 3
// https://drafts.csswg.org/css-images-3/
${helpers.single_keyword("object-fit", "fill contain cover none scale-down",
                         products="gecko", animatable=False)}
