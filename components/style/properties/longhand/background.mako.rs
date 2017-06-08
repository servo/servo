/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />

<% data.new_style_struct("Background", inherited=False) %>

${helpers.predefined_type("background-color", "Color",
    "computed_value::T::transparent()",
    initial_specified_value="SpecifiedValue::transparent()",
    spec="https://drafts.csswg.org/css-backgrounds/#background-color",
    animation_value_type="IntermediateColor",
    ignored_when_colors_disabled=True,
    allow_quirks=True)}

${helpers.predefined_type("background-image", "ImageLayer",
    initial_value="Either::First(None_)",
    initial_specified_value="Either::First(None_)",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-image",
    vector="True",
    animation_value_type="none",
    has_uncacheable_values="True" if product == "gecko" else "False",
    ignored_when_colors_disabled="True")}

% for (axis, direction, initial) in [("x", "Horizontal", "left"), ("y", "Vertical", "top")]:
    ${helpers.predefined_type("background-position-" + axis, "position::" + direction + "Position",
                              initial_value="computed::LengthOrPercentage::zero()",
                              initial_specified_value="SpecifiedValue::initial_specified_value()",
                              spec="https://drafts.csswg.org/css-backgrounds-4/#propdef-background-position-" + axis,
                              animation_value_type="ComputedValue", vector=True, delegate_animate=True)}
% endfor

<%helpers:vector_longhand name="background-repeat" animation_value_type="none"
                          spec="https://drafts.csswg.org/css-backgrounds/#the-background-repeat">
    use std::fmt;
    use style_traits::ToCss;

    define_css_keyword_enum!(RepeatKeyword:
                             "repeat" => Repeat,
                             "space" => Space,
                             "round" => Round,
                             "no-repeat" => NoRepeat);

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum SpecifiedValue {
        RepeatX,
        RepeatY,
        Other(RepeatKeyword, Option<RepeatKeyword>),
    }

    pub mod computed_value {
        pub use super::RepeatKeyword;

        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
        pub struct T(pub RepeatKeyword, pub RepeatKeyword);
    }

    no_viewport_percentage!(SpecifiedValue);

    impl ToCss for computed_value::T {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match (self.0, self.1) {
                (RepeatKeyword::Repeat, RepeatKeyword::NoRepeat) => dest.write_str("repeat-x"),
                (RepeatKeyword::NoRepeat, RepeatKeyword::Repeat) => dest.write_str("repeat-y"),
                (horizontal, vertical) => {
                    try!(horizontal.to_css(dest));
                    if horizontal != vertical {
                        try!(dest.write_str(" "));
                        try!(vertical.to_css(dest));
                    }
                    Ok(())
                },
            }
        }
    }
    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                SpecifiedValue::RepeatX => dest.write_str("repeat-x"),
                SpecifiedValue::RepeatY => dest.write_str("repeat-y"),
                SpecifiedValue::Other(horizontal, vertical) => {
                    try!(horizontal.to_css(dest));
                    if let Some(vertical) = vertical {
                        try!(dest.write_str(" "));
                        try!(vertical.to_css(dest));
                    }
                    Ok(())
                }
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        computed_value::T(RepeatKeyword::Repeat, RepeatKeyword::Repeat)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue::Other(RepeatKeyword::Repeat, None)
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> computed_value::T {
            match *self {
                SpecifiedValue::RepeatX =>
                    computed_value::T(RepeatKeyword::Repeat, RepeatKeyword::NoRepeat),
                SpecifiedValue::RepeatY =>
                    computed_value::T(RepeatKeyword::NoRepeat, RepeatKeyword::Repeat),
                SpecifiedValue::Other(horizontal, vertical) =>
                    computed_value::T(horizontal, vertical.unwrap_or(horizontal))
            }
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            match (computed.0, computed.1) {
                (RepeatKeyword::Repeat, RepeatKeyword::NoRepeat) => SpecifiedValue::RepeatX,
                (RepeatKeyword::NoRepeat, RepeatKeyword::Repeat) => SpecifiedValue::RepeatY,
                (horizontal, vertical) => SpecifiedValue::Other(horizontal, Some(vertical)),
            }
        }
    }

    pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        let ident = input.expect_ident()?;
        match_ignore_ascii_case! { &ident,
            "repeat-x" => Ok(SpecifiedValue::RepeatX),
            "repeat-y" => Ok(SpecifiedValue::RepeatY),
            _ => {
                let horizontal = try!(RepeatKeyword::from_ident(&ident));
                let vertical = input.try(RepeatKeyword::parse).ok();
                Ok(SpecifiedValue::Other(horizontal, vertical))
            }
        }
    }
</%helpers:vector_longhand>

${helpers.single_keyword("background-attachment",
                         "scroll fixed" + (" local" if product == "gecko" else ""),
                         vector=True,
                         gecko_constant_prefix="NS_STYLE_IMAGELAYER_ATTACHMENT",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-attachment",
                         animation_value_type="discrete")}

${helpers.single_keyword("background-clip",
                         "border-box padding-box content-box",
                         extra_gecko_values="text",
                         vector=True, extra_prefixes="webkit",
                         gecko_enum_prefix="StyleGeometryBox",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-clip",
                         animation_value_type="discrete")}

${helpers.single_keyword("background-origin",
                         "padding-box border-box content-box",
                         vector=True, extra_prefixes="webkit",
                         gecko_enum_prefix="StyleGeometryBox",
                         spec="https://drafts.csswg.org/css-backgrounds/#the-background-origin",
                         animation_value_type="discrete")}

${helpers.predefined_type("background-size", "BackgroundSize",
    initial_value="computed::LengthOrPercentageOrAuto::Auto.into()",
    initial_specified_value="specified::LengthOrPercentageOrAuto::Auto.into()",
    spec="https://drafts.csswg.org/css-backgrounds/#the-background-size",
    vector=True,
    animation_value_type="ComputedValue",
    extra_prefixes="webkit")}

// https://drafts.fxtf.org/compositing/#background-blend-mode
${helpers.single_keyword("background-blend-mode",
                         """normal multiply screen overlay darken lighten color-dodge
                            color-burn hard-light soft-light difference exclusion hue
                            saturation color luminosity""",
                         gecko_constant_prefix="NS_STYLE_BLEND",
                         vector=True, products="gecko", animation_value_type="discrete",
                         spec="https://drafts.fxtf.org/compositing/#background-blend-mode")}
