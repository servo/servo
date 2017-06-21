/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Keyword, Method, PHYSICAL_SIDES, ALL_SIDES, maybe_moz_logical_alias %>

<% data.new_style_struct("Border", inherited=False,
                   additional_methods=[Method("border_" + side + "_has_nonzero_width",
                                              "bool") for side in ["top", "right", "bottom", "left"]]) %>
<%
    def maybe_logical_spec(side, kind):
        if side[1]: # if it is logical
            return "https://drafts.csswg.org/css-logical-props/#propdef-border-%s-%s" % (side[0], kind)
        else:
            return "https://drafts.csswg.org/css-backgrounds/#border-%s-%s" % (side[0], kind)
%>
% for side in ALL_SIDES:
    ${helpers.predefined_type("border-%s-color" % side[0], "Color",
                              "computed_value::T::currentcolor()",
                              alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-color"),
                              spec=maybe_logical_spec(side, "color"),
                              animation_value_type="IntermediateColor",
                              logical=side[1],
                              allow_quirks=not side[1],
                              ignored_when_colors_disabled=True)}

    ${helpers.predefined_type("border-%s-style" % side[0], "BorderStyle",
                              "specified::BorderStyle::none",
                              need_clone=True,
                              alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-style"),
                              spec=maybe_logical_spec(side, "style"),
                              animation_value_type="none", logical=side[1])}

    ${helpers.predefined_type("border-%s-width" % side[0],
                              "BorderSideWidth",
                              "Au::from_px(3)",
                              computed_type="::app_units::Au",
                              alias=maybe_moz_logical_alias(product, side, "-moz-border-%s-width"),
                              spec=maybe_logical_spec(side, "width"),
                              animation_value_type="ComputedValue",
                              logical=side[1],
                              allow_quirks=not side[1])}
% endfor

${helpers.gecko_keyword_conversion(Keyword('border-style',
                                   "none solid double dotted dashed hidden groove ridge inset outset"),
                                   type="::values::specified::BorderStyle")}

// FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
% for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
    ${helpers.predefined_type("border-" + corner + "-radius", "BorderCornerRadius",
                              "computed::LengthOrPercentage::zero().into()",
                              "parse", extra_prefixes="webkit",
                              spec="https://drafts.csswg.org/css-backgrounds/#border-%s-radius" % corner,
                              boxed=True,
                              animation_value_type="ComputedValue")}
% endfor

/// -moz-border-*-colors: color, string, enum, none, inherit/initial
/// These non-spec properties are just for Gecko (Stylo) internal use.
% for side in PHYSICAL_SIDES:
    <%helpers:longhand name="-moz-border-${side}-colors" animation_value_type="none"
                       spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-border-*-colors)"
                       products="gecko"
                       ignored_when_colors_disabled="True">
        use std::fmt;
        use style_traits::ToCss;
        use values::specified::RGBAColor;
        no_viewport_percentage!(SpecifiedValue);

        pub mod computed_value {
            use cssparser::RGBA;
            #[derive(Debug, Clone, PartialEq)]
            #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
            pub struct T(pub Option<Vec<RGBA>>);
        }

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub enum SpecifiedValue {
            None,
            Colors(Vec<RGBAColor>),
        }

        impl ToCss for computed_value::T {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match self.0 {
                    None => return dest.write_str("none"),
                    Some(ref vec) => {
                        let mut first = true;
                        for ref color in vec {
                            if !first {
                                dest.write_str(" ")?;
                            }
                            first = false;
                            color.to_css(dest)?
                        }
                        Ok(())
                    }
                }
            }
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
                match *self {
                    SpecifiedValue::None => return dest.write_str("none"),
                    SpecifiedValue::Colors(ref vec) => {
                        let mut first = true;
                        for ref color in vec {
                            if !first {
                                dest.write_str(" ")?;
                            }
                            first = false;
                            color.to_css(dest)?
                        }
                        Ok(())
                    }
                }
            }
        }

        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::T(None)
        }

        #[inline] pub fn get_initial_specified_value() -> SpecifiedValue {
            SpecifiedValue::None
        }

        impl ToComputedValue for SpecifiedValue {
            type ComputedValue = computed_value::T;

            #[inline]
            fn to_computed_value(&self, context: &Context) -> computed_value::T {
                match *self {
                    SpecifiedValue::Colors(ref vec) => {
                        computed_value::T(Some(vec.iter()
                                                  .map(|c| c.to_computed_value(context))
                                                  .collect()))
                    },
                    SpecifiedValue::None => {
                        computed_value::T(None)
                    }
                }
            }
            #[inline]
            fn from_computed_value(computed: &computed_value::T) -> Self {
                match *computed {
                    computed_value::T(Some(ref vec)) => {
                        SpecifiedValue::Colors(vec.iter()
                                                  .map(ToComputedValue::from_computed_value)
                                                  .collect())
                    },
                    computed_value::T(None) => {
                        SpecifiedValue::None
                    }
                }
            }
        }

        #[inline]
        pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                             -> Result<SpecifiedValue, ParseError<'i>> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue::None)
            }

            let mut result = Vec::new();
            while let Ok(value) = input.try(|i| RGBAColor::parse(context, i)) {
                result.push(value);
            }

            if !result.is_empty() {
                Ok(SpecifiedValue::Colors(result))
            } else {
                Err(StyleParseError::UnspecifiedError.into())
            }
        }
    </%helpers:longhand>
% endfor

${helpers.single_keyword("box-decoration-break", "slice clone",
                         gecko_enum_prefix="StyleBoxDecorationBreak",
                         gecko_inexhaustive=True,
                         spec="https://drafts.csswg.org/css-break/#propdef-box-decoration-break",
                         products="gecko", animation_value_type="discrete")}

${helpers.single_keyword("-moz-float-edge", "content-box margin-box",
                         gecko_ffi_name="mFloatEdge",
                         gecko_enum_prefix="StyleFloatEdge",
                         gecko_inexhaustive=True,
                         products="gecko",
                         spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-float-edge)",
                         animation_value_type="discrete")}

${helpers.predefined_type("border-image-source", "ImageLayer",
    initial_value="Either::First(None_)",
    initial_specified_value="Either::First(None_)",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-image",
    vector=False,
    animation_value_type="none",
    has_uncacheable_values=False,
    boxed="True")}

${helpers.predefined_type("border-image-outset", "LengthOrNumberRect",
    parse_method="parse_non_negative",
    initial_value="computed::LengthOrNumber::zero().into()",
    initial_specified_value="specified::LengthOrNumber::zero().into()",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-outset",
    animation_value_type="none",
    boxed=True)}

<%helpers:longhand name="border-image-repeat" animation_value_type="discrete"
                   spec="https://drafts.csswg.org/css-backgrounds/#border-image-repeat">
    use style_traits::ToCss;

    no_viewport_percentage!(SpecifiedValue);

    pub mod computed_value {
        pub use super::RepeatKeyword;

        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        #[derive(Debug, Clone, PartialEq, ToCss)]
        pub struct T(pub RepeatKeyword, pub RepeatKeyword);
    }

    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    #[derive(Debug, Clone, PartialEq, ToCss)]
    pub struct SpecifiedValue(pub RepeatKeyword,
                              pub Option<RepeatKeyword>);

    define_css_keyword_enum!(RepeatKeyword:
                             "stretch" => Stretch,
                             "repeat" => Repeat,
                             "round" => Round,
                             "space" => Space);

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(RepeatKeyword::Stretch, RepeatKeyword::Stretch)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(RepeatKeyword::Stretch, None)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            computed_value::T(self.0, self.1.unwrap_or(self.0))
        }
        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(computed.0, Some(computed.1))
        }
    }

    pub fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<SpecifiedValue, ParseError<'i>> {
        let first = RepeatKeyword::parse(input)?;
        let second = input.try(RepeatKeyword::parse).ok();

        Ok(SpecifiedValue(first, second))
    }
</%helpers:longhand>

${helpers.predefined_type("border-image-width", "BorderImageWidth",
    initial_value="computed::BorderImageSideWidth::one().into()",
    initial_specified_value="specified::BorderImageSideWidth::one().into()",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-width",
    animation_value_type="none",
    boxed=True)}

${helpers.predefined_type("border-image-slice", "BorderImageSlice",
    initial_value="computed::NumberOrPercentage::Percentage(computed::Percentage(1.)).into()",
    initial_specified_value="specified::NumberOrPercentage::Percentage(specified::Percentage(1.)).into()",
    spec="https://drafts.csswg.org/css-backgrounds/#border-image-slice",
    animation_value_type="none",
    boxed=True)}
