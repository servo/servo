/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import Method %>

<% data.new_style_struct("Outline",
                         inherited=False,
                         additional_methods=[Method("outline_has_nonzero_width", "bool")]) %>

// TODO(pcwalton): `invert`
${helpers.predefined_type("outline-color", "CSSColor", "computed::CSSColor::CurrentColor",
                          initial_specified_value="specified::CSSColor::currentcolor()",
                          animation_value_type="IntermediateColor", complex_color=True, need_clone=True,
                          spec="https://drafts.csswg.org/css-ui/#propdef-outline-color")}

<%helpers:longhand name="outline-style" need_clone="True" animation_value_type="none"
                   spec="https://drafts.csswg.org/css-ui/#propdef-outline-style">

    use std::fmt;
    use style_traits::ToCss;
    use values::specified::BorderStyle;
    use values::computed::ComputedValueAsSpecified;

    pub type SpecifiedValue = Either<Auto, BorderStyle>;

    impl SpecifiedValue {
        #[inline]
        pub fn none_or_hidden(&self) -> bool {
            match *self {
                Either::First(ref _auto) => false,
                Either::Second(ref border_style) => border_style.none_or_hidden()
            }
        }
    }

    #[inline]
    pub fn get_initial_value() -> computed_value::T {
        Either::Second(BorderStyle::none)
    }

    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        Either::Second(BorderStyle::none)
    }

    pub mod computed_value {
        pub type T = super::SpecifiedValue;
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        SpecifiedValue::parse(context, input)
            .and_then(|result| {
                if let Either::Second(BorderStyle::hidden) = result {
                    // The outline-style property accepts the same values as
                    // border-style, except that 'hidden' is not a legal outline
                    // style.
                    Err(())
                } else {
                    Ok(result)
                }
            })
    }
</%helpers:longhand>

<%helpers:longhand name="outline-width" animation_value_type="ComputedValue"
                   spec="https://drafts.csswg.org/css-ui/#propdef-outline-width">
    use app_units::Au;
    use std::fmt;
    use style_traits::ToCss;
    use values::HasViewportPercentage;

    impl ToCss for SpecifiedValue {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
        specified::parse_border_width(context, input).map(SpecifiedValue)
    }

    impl HasViewportPercentage for SpecifiedValue {
        fn has_viewport_percentage(&self) -> bool {
            let &SpecifiedValue(ref length) = self;
            length.has_viewport_percentage()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct SpecifiedValue(pub specified::Length);
    pub mod computed_value {
        use app_units::Au;
        pub type T = Au;
    }

    pub use super::border_top_width::get_initial_value;
    #[inline]
    pub fn get_initial_specified_value() -> SpecifiedValue {
        SpecifiedValue(specified::Length::NoCalc(specified::NoCalcLength::medium()))
    }

    impl ToComputedValue for SpecifiedValue {
        type ComputedValue = computed_value::T;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> computed_value::T {
            self.0.to_computed_value(context)
        }

        #[inline]
        fn from_computed_value(computed: &computed_value::T) -> Self {
            SpecifiedValue(ToComputedValue::from_computed_value(computed))
        }
    }
</%helpers:longhand>

// The -moz-outline-radius-* properties are non-standard and not on a standards track.
// TODO: Should they animate?
% for corner in ["topleft", "topright", "bottomright", "bottomleft"]:
    ${helpers.predefined_type("-moz-outline-radius-" + corner, "BorderRadiusSize",
        "computed::BorderRadiusSize::zero()",
        "parse", products="gecko",
        boxed=True,
        animation_value_type="none",
        spec="Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-outline-radius)")}
% endfor

${helpers.predefined_type("outline-offset", "Length", "Au(0)", products="servo gecko",
                          animation_value_type="ComputedValue",
                          spec="https://drafts.csswg.org/css-ui/#propdef-outline-offset")}
